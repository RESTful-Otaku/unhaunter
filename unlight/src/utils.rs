use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_platform::collections::HashSet;
use ndarray::Array3;
use std::collections::VecDeque;
use uncore::{
    behavior::{Behavior, TileState},
    components::board::{boardposition::BoardPosition, position::Position},
    resources::board_data::BoardData,
    types::board::{
        fielddata::LightFieldData,
        prebaked_lighting_data::{WaveEdge, WaveEdgeData},
    },
};

/// Checks if a position is within the board boundaries
pub fn is_in_bounds(pos: (i64, i64, i64), map_size: (usize, usize, usize)) -> bool {
    pos.0 >= 0
        && pos.1 >= 0
        && pos.2 >= 0
        && pos.0 < map_size.0 as i64
        && pos.1 < map_size.1 as i64
        && pos.2 < map_size.2 as i64
}

/// Helper function to check if there are active light sources nearby
pub fn has_active_light_nearby(
    bf: &BoardData,
    active_source_ids: &HashSet<u32>,
    i: usize,
    j: usize,
    k: usize,
) -> bool {
    // Check immediate neighbours plus the current position
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let nx = i as i64 + dx;
                let ny = j as i64 + dy;
                let nz = k as i64 + dz;

                // Skip if out of bounds
                if !is_in_bounds((nx, ny, nz), bf.map_size) {
                    continue;
                }

                let pos = (nx as usize, ny as usize, nz as usize);
                let prebaked_data = &bf.prebaked_lighting[pos];

                if let Some(source_id) = prebaked_data.light_info.source_id {
                    if active_source_ids.contains(&source_id) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Determines if a light is currently active based on its position and behavior
pub fn is_light_active(pos: &BoardPosition, behaviors: &HashMap<BoardPosition, &Behavior>) -> bool {
    if let Some(behavior) = behaviors.get(pos) {
        behavior.p.light.light_emission_enabled
    } else {
        false
    }
}

/// Blend two colors based on their intensity
pub fn blend_colors(
    c1: (f32, f32, f32),
    lux1: f32,
    c2: (f32, f32, f32),
    lux2: f32,
) -> (f32, f32, f32) {
    let total_lux = lux1 + lux2;
    if total_lux <= 0.0 {
        return (1.0, 1.0, 1.0);
    }
    (
        (c1.0 * lux1 + c2.0 * lux2) / total_lux,
        (c1.1 * lux1 + c2.1 * lux2) / total_lux,
        (c1.2 * lux1 + c2.2 * lux2) / total_lux,
    )
}

/// Identifies active light sources in the scene
pub fn identify_active_light_sources(
    bf: &BoardData,
    qt: &Query<(&Position, &Behavior)>,
) -> HashSet<u32> {
    let mut active_source_ids = HashSet::new();

    for (entity, ndidx) in &bf.prebaked_metadata.light_sources {
        let Ok((_pos, behavior)) = qt.get(*entity) else {
            continue;
        };

        if behavior.p.light.light_emission_enabled {
            if let Some(source_id) = bf.prebaked_lighting[*ndidx].light_info.source_id {
                active_source_ids.insert(source_id);
            }
        }
    }
    // info!(
    //     "Active light sources: {}/{} (prebaked) ",
    //     active_source_ids.len(),
    //     bf.prebaked_lighting
    //         .iter()
    //         .filter(|d| d.light_info.source_id.is_some())
    //         .count(),
    // );

    active_source_ids
}

/// Apply prebaked light contributions from active sources
pub fn apply_prebaked_contributions(
    active_source_ids: &HashSet<u32>,
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
) -> usize {
    let mut tiles_lit = 0;
    let mut v_active = vec![false; bf.prebaked_propagation.len()];
    for source_id in active_source_ids {
        v_active[*source_id as usize] = true;
    }
    // Apply light from active prebaked sources to the lighting field
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        let pos_idx = (i, j, k);

        // Get the source ID (if any)
        if let Some(source_id) = prebaked_data.light_info.source_id {
            // Only apply if this source is currently active
            if v_active[source_id as usize] {
                let lux = prebaked_data.light_info.lux;

                // Apply light to this position
                lfs[pos_idx].lux = lux;
                lfs[pos_idx].color = prebaked_data.light_info.color;
                tiles_lit += 1;
            }
        }
    }

    // info!("Applied prebaked light: {} tiles lit", tiles_lit);
    tiles_lit
}

/// Update final exposure settings and log statistics
pub fn update_exposure_and_stats(bf: &mut BoardData, lfs: &Array3<LightFieldData>) {
    let _tiles_with_light = lfs.iter().filter(|x| x.lux > 0.0).count();
    let total_tiles = bf.map_size.0 * bf.map_size.1 * bf.map_size.2;
    let _avg_lux = lfs.iter().map(|x| x.lux).sum::<f32>() / total_tiles as f32;
    let _max_lux = lfs.iter().map(|x| x.lux).fold(0.0, f32::max);

    // info!(
    //     "Light field stats: {}/{} tiles lit ({:.2}%), avg: {:.6}, max: {:.6}",
    //     tiles_with_light,
    //     total_tiles,
    //     (tiles_with_light as f32 / total_tiles as f32) * 100.0,
    //     avg_lux,
    //     max_lux
    // );

    // Calculate exposure
    let total_lux: f32 = lfs.iter().map(|x| x.lux).sum();
    let count = total_tiles as f32;
    let avg_lux = total_lux / count;
    bf.exposure_lux = (avg_lux + 2.0) / 2.0;
    bf.light_field = lfs.clone();

    // info!("Final exposure_lux set to: {}", bf.exposure_lux);
}

/// Collects information about door states from entity behaviors
pub fn collect_door_states(
    bf: &BoardData,
    qt: &Query<(&Position, &Behavior)>,
) -> HashMap<(usize, usize, usize), bool> {
    let mut door_states = HashMap::new();
    for entity in &bf.prebaked_metadata.doors {
        if let Ok((pos, behavior)) = qt.get(*entity) {
            let board_pos = pos.to_board_position();
            let idx = board_pos.ndidx();
            let is_open = behavior.state() == TileState::Open;

            // Store the door's open state (true if open, false if closed)
            door_states.insert(idx, is_open);
        }
    }

    // info!("Collected {} door states", door_states.len());
    door_states
}

/// Finds wave edge tiles for continuing light propagation
pub fn find_wave_edge_tiles(bf: &BoardData, active_source_ids: &HashSet<u32>) -> Vec<WaveEdgeData> {
    let mut wave_edges = Vec::new();

    // Find all wave edge tiles where light propagation can continue
    for ((i, j, k), prebaked_data) in bf.prebaked_lighting.indexed_iter() {
        // Skip if not a wave edge
        let Some(wave_edge) = &prebaked_data.wave_edge else {
            continue;
        };

        // Skip if no source info
        let Some(source_id) = prebaked_data.light_info.source_id else {
            continue;
        };

        // Skip if source is not active
        if !active_source_ids.contains(&source_id) {
            continue;
        }

        // Add to wave edges (whether or not it's near a door)
        let pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: k as i64,
        };

        wave_edges.push(WaveEdgeData {
            position: pos,
            source_id,
            lux: prebaked_data.light_info.lux,
            color: prebaked_data.light_info.color,
            wave_edge: wave_edge.clone(),
        });
    }

    // info!("Found {} wave edge tiles for propagation", wave_edges.len());
    wave_edges
}

fn apply_iir_filter(
    current_value: (f32, f32, f32),
    new_value: (f32, f32, f32),
    factor: f32,
) -> (f32, f32, f32) {
    (
        current_value.0 * factor + new_value.0 * (1.0 - factor),
        current_value.1 * factor + new_value.1 * (1.0 - factor),
        current_value.2 * factor + new_value.2 * (1.0 - factor),
    )
}

/// Propagates light from wave edge tiles past dynamic objects
pub fn propagate_from_wave_edges(
    bf: &BoardData,
    lfs: &mut Array3<LightFieldData>,
    active_source_ids: &HashSet<u32>,
) -> usize {
    // New struct to track position history for turn detection
    #[derive(Clone)]
    struct InternalWaveEdge {
        position: BoardPosition,
        wave_edge: WaveEdge,
        source_id: u32,
        color: (f32, f32, f32),
    }

    let mut queue = VecDeque::with_capacity(4096);
    let mut propagation_count = 0;
    let mut _stair_propagation_count = 0;

    // Define directions for propagation
    let directions = [(0, -1, 0), (0, 1, 0), (-1, 0, 0), (1, 0, 0)];

    // IIR factors (adjust these to control the "smoothness")
    const IIR_FACTOR_1: f32 = 0.8; // First level of smoothing
    const IIR_FACTOR_2: f32 = 0.8; // Second level of smoothing

    // Log which source IDs are active for debugging
    // info!("Active source IDs for propagation: {:?}", active_source_ids);

    // Track stair wave edges
    let mut _stair_wave_edge_count = 0;

    // Add all wave edges to the queue
    for edge_data in bf.prebaked_wave_edges.iter() {
        if !active_source_ids.contains(&edge_data.source_id) {
            continue;
        }

        // Check if this is a stair wave edge (source_id == 0)
        if edge_data.source_id == 0 {
            _stair_wave_edge_count += 1;
            // info!(
            //     "Adding stair wave edge at ({}, {}, {}) with lux: {}",
            //     edge_data.position.x, edge_data.position.y, edge_data.position.z, edge_data.lux
            // );
        }

        queue.push_back(InternalWaveEdge {
            position: edge_data.position.clone(),
            wave_edge: edge_data.wave_edge.clone(),
            source_id: edge_data.source_id,
            color: edge_data.color,
        });
    }

    // info!(
    //     "Added {} stair wave edges to propagation queue",
    //     stair_wave_edge_count
    // );

    // Process queue using BFS
    while let Some(edge_data) = queue.pop_front() {
        let pos = edge_data.position;
        let max_lux_possible = edge_data.wave_edge.src_light_lux
            / (edge_data.wave_edge.distance_travelled * edge_data.wave_edge.distance_travelled);

        // If light is too low, skip
        if max_lux_possible < 0.0000001 {
            continue;
        }

        // Special handling for stair wave edges (source_id == 0)
        let is_stair_edge = edge_data.source_id == 0;

        // Log propagation from stair wave edges
        // if is_stair_edge {
        //     info!(
        //         "Processing stair wave edge at ({}, {}, {}), max lux possible: {}",
        //         pos.x, pos.y, pos.z, max_lux_possible
        //     );
        // }

        // For stair wave edges, we don't use prebaked propagation directions
        // For regular wave edges, we check the prebaked propagation directions
        let allowed_directions = if is_stair_edge {
            // For stair wave edges, allow all directions
            [true, true, true, true]
        } else {
            // For regular wave edges, use prebaked directions
            match bf
                .prebaked_propagation
                .get(edge_data.source_id as usize)
                .and_then(|arr| arr.get((pos.x as usize, pos.y as usize)))
            {
                Some(dirs) => *dirs,
                None => {
                    if is_stair_edge {
                        // info!("No prebaked propagation directions for stair wave edge, skipping");
                    }
                    continue;
                }
            }
        };

        // Process each neighbour direction
        for (dir_idx, &(dx, dy, dz)) in directions.iter().enumerate() {
            // Skip if not allowed in this direction
            if !is_stair_edge && !allowed_directions[dir_idx] {
                continue;
            }

            let nx = pos.x + dx;
            let ny = pos.y + dy;
            let nz = pos.z + dz;

            // Skip if out of bounds
            if !is_in_bounds((nx, ny, nz), bf.map_size) {
                continue;
            }

            let neighbour_pos = BoardPosition {
                x: nx,
                y: ny,
                z: nz,
            };

            let neighbour_idx = neighbour_pos.ndidx();

            // Stop checking early if this neighbour is already too bright
            if lfs[neighbour_idx].lux > max_lux_possible * 4.0 {
                continue;
            }

            // For regular wave edges, skip if neighbour was already in prebaked data
            // For stair wave edges, don't skip
            if !is_stair_edge
                && Some(edge_data.source_id)
                    == bf.prebaked_lighting[neighbour_idx].light_info.source_id
            {
                continue;
            }

            // Check collision data
            let collision = &bf.collision_field[neighbour_idx];

            // Update wave edge position using IIR filter
            let new_pos_f32 = (nx as f32, ny as f32, nz as f32);

            // Update the current position.
            let mut new_wave_edge = edge_data.wave_edge.clone();
            new_wave_edge.current_pos = new_pos_f32;

            // Apply the first IIR filter to update the mean position.
            new_wave_edge.iir_mean_pos =
                apply_iir_filter(new_wave_edge.iir_mean_pos, new_pos_f32, IIR_FACTOR_1);

            // Apply the second IIR filter to update the mean of the mean position.
            new_wave_edge.iir_mean_iir_mean_pos = apply_iir_filter(
                new_wave_edge.iir_mean_iir_mean_pos,
                new_wave_edge.iir_mean_pos,
                IIR_FACTOR_2,
            );

            let mut turn_penalty = {
                // old_dir is now: from iir_mean_iir_mean_pos to iir_mean_pos
                let old_dir = (
                    new_wave_edge.iir_mean_pos.0 - new_wave_edge.iir_mean_iir_mean_pos.0,
                    new_wave_edge.iir_mean_pos.1 - new_wave_edge.iir_mean_iir_mean_pos.1,
                    new_wave_edge.iir_mean_pos.2 - new_wave_edge.iir_mean_iir_mean_pos.2,
                );

                // recent_dir is now: from iir_mean_pos to current_pos
                let recent_dir = (
                    new_wave_edge.current_pos.0 - new_wave_edge.iir_mean_pos.0,
                    new_wave_edge.current_pos.1 - new_wave_edge.iir_mean_pos.1,
                    new_wave_edge.current_pos.2 - new_wave_edge.iir_mean_pos.2,
                );

                // Normalize vectors and compute dot product
                let old_len =
                    (old_dir.0 * old_dir.0 + old_dir.1 * old_dir.1 + old_dir.2 * old_dir.2).sqrt();
                let recent_len = (recent_dir.0 * recent_dir.0
                    + recent_dir.1 * recent_dir.1
                    + recent_dir.2 * recent_dir.2)
                    .sqrt();

                if old_len > 0.0 && recent_len > 0.0 {
                    let old_norm = (
                        old_dir.0 / old_len,
                        old_dir.1 / old_len,
                        old_dir.2 / old_len,
                    );
                    let recent_norm = (
                        recent_dir.0 / recent_len,
                        recent_dir.1 / recent_len,
                        recent_dir.2 / recent_len,
                    );
                    let dot_product = old_norm.0 * recent_norm.0
                        + old_norm.1 * recent_norm.1
                        + old_norm.2 * recent_norm.2;

                    let dot_product = (dot_product + 0.01).clamp(-1.0, 1.0);
                    const TURN_FACTOR: f32 = 0.6; // Adjust for turn penalty
                    1.0 + (1.0 - dot_product) * TURN_FACTOR
                } else {
                    1.0
                }
            };
            if max_lux_possible > 0.2 {
                turn_penalty += 0.1;
            }

            // Use higher transparency for stair wave edges
            let transparency = if is_stair_edge {
                if collision.see_through {
                    0.9 // Higher transparency for stairs
                } else {
                    0.1 // Still need some penalty for walls
                }
            } else if collision.player_free && collision.see_through && !collision.is_dynamic {
                0.98 / turn_penalty.min(1.5)
            } else if collision.see_through {
                0.4
            } else {
                0.05
            };

            let src_light_lux = edge_data.wave_edge.src_light_lux * transparency;
            let distance_travelled = edge_data.wave_edge.distance_travelled;

            // Apply the turn penalty to the light intensity
            let new_lux = src_light_lux / (distance_travelled * distance_travelled);

            new_wave_edge.distance_travelled += 1.0;
            new_wave_edge.src_light_lux = src_light_lux;

            // Skip propagating if the contribution is too small
            if lfs[neighbour_idx].lux > new_lux * 5.0 {
                continue;
            } else if lfs[neighbour_idx].lux > new_lux * 2.0 {
                new_wave_edge.src_light_lux /= 1.1;
            }

            // Update light field for neighbour
            if lfs[neighbour_idx].lux > 0.0 {
                lfs[neighbour_idx].color = blend_colors(
                    lfs[neighbour_idx].color,
                    lfs[neighbour_idx].lux,
                    edge_data.color,
                    new_lux,
                );
            } else {
                lfs[neighbour_idx].color = edge_data.color;
            }

            lfs[neighbour_idx].lux += new_lux;

            // Log when adding light to a cell from a stair wave edge
            if is_stair_edge {
                // info!(
                //     "  Stair light propagated to ({}, {}, {}): added lux {} (total now: {})",
                //     nx, ny, nz, new_lux, lfs[neighbour_idx].lux
                // );
                _stair_propagation_count += 1;
            }

            // Add neighbour to queue with updated history
            queue.push_back(InternalWaveEdge {
                position: neighbour_pos,
                wave_edge: new_wave_edge,
                source_id: edge_data.source_id,
                color: edge_data.color,
            });

            propagation_count += 1;
        }
    }

    // info!(
    //     "Light propagation: {} total steps, {} from stairs",
    //     propagation_count, stair_propagation_count
    // );
    propagation_count
}

/// Propagates light through stairs between floors
pub fn propagate_through_stairs(bf: &BoardData, lfs: &mut Array3<LightFieldData>) -> usize {
    let mut propagation_count = 0;
    const STAIR_PROPAGATION: f32 = 0.99;

    // Process all stair tiles
    for ((i, j, k), collision) in bf.collision_field.indexed_iter() {
        // Only process stairs
        if collision.stair_offset == 0 {
            continue;
        }

        let pos = (i, j, k);
        let stair_lux = lfs[pos].lux;
        let stair_color = lfs[pos].color;

        // Determine target position (up or down based on stair_offset)
        let target_z = k as i64 + collision.stair_offset as i64;
        if target_z < 0 || target_z >= bf.map_size.2 as i64 {
            continue; // Out of bounds
        }

        let target_pos = (i, j, target_z as usize);

        // Only update if we're bringing more light to the target
        if lfs[target_pos].lux < stair_lux * STAIR_PROPAGATION {
            // Update the target's light
            let old_lux = lfs[target_pos].lux;

            if old_lux > 0.0 {
                // Blend with existing light
                lfs[target_pos].color = blend_colors(
                    lfs[target_pos].color,
                    old_lux,
                    stair_color,
                    stair_lux * STAIR_PROPAGATION,
                );
            } else {
                // No existing light
                lfs[target_pos].color = stair_color;
            }

            // Set new light value
            lfs[target_pos].lux = stair_lux * STAIR_PROPAGATION;
            propagation_count += 1;
        }
    }

    // info!(
    //     "Stair light propagation: {} propagations",
    //     propagation_count
    // );
    propagation_count
}

/// Creates wave edges at stair connections between floors to allow light propagation
pub fn create_stair_wave_edges(bf: &BoardData, lfs: &Array3<LightFieldData>) -> Vec<WaveEdgeData> {
    let mut wave_edges = Vec::new();
    let mut _stair_tiles_found = 0;

    // Process all stair tiles
    for ((i, j, k), collision) in bf.collision_field.indexed_iter() {
        // Only process stairs
        if collision.stair_offset == 0 {
            continue;
        }

        _stair_tiles_found += 1;
        // info!(
        //     "Found stair at ({}, {}, {}) with offset {}",
        //     i, j, k, collision.stair_offset
        // );

        let pos = (i, j, k);
        let stair_lux = lfs[pos].lux;

        // Log stair light info
        // info!("  Stair has lux: {}", stair_lux);

        // Skip if no light
        if stair_lux <= 0.0 {
            // info!("  Skipped: no light on stair");
            continue;
        }

        let stair_color = lfs[pos].color;

        // Determine target position based on stair offset
        let target_z = k as i64 + collision.stair_offset as i64;
        if target_z < 0 || target_z >= bf.map_size.2 as i64 {
            // info!("  Skipped: target position out of bounds");
            continue; // Out of bounds
        }

        let target_pos = (i, j, target_z as usize);
        let target_lux = lfs[target_pos].lux;
        // info!("  Target at floor {}: has lux {}", target_z, target_lux);

        // Only create wave edge if we can bring more light
        if stair_lux <= target_lux {
            // info!("  Skipped: target already has more light than stair");
            continue;
        }

        // Create a wave edge at the target position
        let source_pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: k as i64,
        };
        let target_board_pos = BoardPosition {
            x: i as i64,
            y: j as i64,
            z: target_z,
        };

        // Calculate the distance between floors (1.0 for adjacent floors)
        let distance = 1.0;

        // Create a wave edge with the same relative intensity
        let wave_edge = WaveEdge {
            src_light_lux: stair_lux * (distance * distance), // Compensate for distance attenuation
            distance_travelled: distance,
            current_pos: (
                target_board_pos.x as f32,
                target_board_pos.y as f32,
                target_board_pos.z as f32,
            ),
            iir_mean_pos: (
                target_board_pos.x as f32,
                target_board_pos.y as f32,
                target_board_pos.z as f32,
            ),
            iir_mean_iir_mean_pos: (
                source_pos.x as f32,
                source_pos.y as f32,
                source_pos.z as f32,
            ),
        };

        // We don't have a specific source ID for this light, so we'll use a dummy ID
        // that doesn't conflict with existing sources
        let dummy_source_id = 0; // Special ID for stair propagation

        // info!(
        //     "  Creating wave edge: src_lux={}, distance={}, src_id={}",
        //     wave_edge.src_light_lux, wave_edge.distance_travelled, dummy_source_id
        // );

        wave_edges.push(WaveEdgeData {
            position: target_board_pos,
            source_id: dummy_source_id,
            lux: stair_lux,
            color: stair_color,
            wave_edge,
        });
    }

    // info!(
    //     "Stairs: found {} stair tiles, created {} wave edges",
    //     stair_tiles_found,
    //     wave_edges.len()
    // );
    wave_edges
}
