use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_platform::collections::HashMap;
use ndarray::{Array3, s};
use rand::Rng;
use uncore::behavior::Behavior;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::chunk::{CellIterator, ChunkIterator};
use uncore::components::board::mapcolor::MapColor;
use uncore::components::board::position::Position;
use uncore::components::game::GameSprite;
use uncore::components::game_config::GameConfig;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::components::player::Hiding;
use uncore::components::player_sprite::PlayerSprite;
use uncore::components::sprite_type::SpriteType;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::events::loadlevel::LevelReadyEvent;
use uncore::metric_recorder::SendMetric;
use uncore::random_seed;
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;
use uncore::resources::visibility_data::VisibilityData;
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use unstd::plugins::board::rebuild_collision_data;

use crate::components::{MiasmaHazard, MiasmaSprite};
use crate::metrics;
use crate::resources::MiasmaConfig;

fn initialize_miasma(
    mut board_data: ResMut<BoardData>,
    roomdb: Res<RoomDB>,
    config: Res<MiasmaConfig>,
    mut level_ready: EventReader<LevelReadyEvent>,
    qt: Query<(Entity, &Position, &Behavior)>,
) {
    // Only run on LevelLoadedEvent
    if level_ready.read().next().is_none() {
        return;
    }
    warn!("Miasma Init");
    rebuild_collision_data(&mut board_data, &qt);

    board_data.miasma.room_modifiers.clear();

    let mut rng = random_seed::rng();
    let collision_field = board_data.collision_field.clone();

    for (p, cfield) in collision_field.indexed_iter() {
        let board_position = BoardPosition::from_ndidx(p);
        let opt_room_id = roomdb.room_tiles.get(&board_position);

        // 1. Get or Insert Room Modifier:
        let mut modifier = if let Some(room_id) = opt_room_id {
            *board_data
                .miasma
                .room_modifiers
                .entry(room_id.clone()) // Try to get the modifier for this room_id.
                .or_insert_with(|| rng.random_range(0.4..=2.9)) // If not found, create one.
        } else {
            0.0
        };
        if !cfield.player_free {
            modifier = 0.0;
        }

        // 2. Set Initial Pressure:
        board_data.miasma.pressure_field[board_position.ndidx()] =
            config.initial_room_pressure * modifier * rng.random_range(0.9..=1.1);
    }
    warn!("Done: Miasma Init");
}

fn spawn_miasma(
    time: Res<Time>,
    vf: Res<VisibilityData>,
    mut q_miasma: Query<(Entity, &mut MiasmaSprite)>,
    gc: Res<GameConfig>,
    qp: Query<(&Position, &PlayerSprite)>,
    handles: Res<GameAssets>,
    board_data: Res<BoardData>,
    mut commands: Commands,
) {
    let measure = metrics::SPAWN_MIASMA.time_measure();
    const THRESHOLD: f32 = 0.000001;
    const DIST_FACTOR: f32 = 0.00001;
    const MIASMA_TARGET_SPRITE_COUNT: usize = 3;
    let mut rng = random_seed::rng();
    let dt = time.delta_secs();

    if board_data.map_size.0 == 0 {
        return;
    }
    // Find the active player's position
    let Some(player_pos) = qp.iter().find_map(|(pos, player)| {
        if player.id == gc.player_id {
            Some(*pos)
        } else {
            None
        }
    }) else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    if vf.visibility_field.dim() != board_data.collision_field.dim() {
        // If the visibility field hasn't updated to the same size, skip processing.
        // This happens on map load and takes 1-2 frames to stabilize.
        return;
    }

    let mut count: HashMap<BoardPosition, usize> = HashMap::new();
    for (entity, mut miasma_sprite) in q_miasma.iter_mut() {
        if miasma_sprite.despawn {
            miasma_sprite.life -= dt * 2.0;
        }
        if miasma_sprite.life < -2.0 {
            miasma_sprite.life = -2.0;
            commands.entity(entity).despawn();
        }
        if miasma_sprite.despawn {
            continue;
        }
        let bpos = miasma_sprite.base_position.to_board_position();
        let Some(pressure) = board_data.miasma.pressure_field.get(bpos.ndidx()) else {
            miasma_sprite.despawn = true;
            continue;
        };
        miasma_sprite.life -= dt / 10.0;
        if miasma_sprite.life < 0.02 {
            miasma_sprite.despawn = true;
            continue;
        }
        let player_dst2 = player_pos.distance2(&miasma_sprite.base_position);

        let vis = vf.visibility_field[bpos.ndidx()] + DIST_FACTOR / player_dst2;
        let target_count =
            ((pressure.cbrt() / 3.1 + 0.1).min(1.0) * MIASMA_TARGET_SPRITE_COUNT as f32) as usize;

        let pos_count = count.entry(bpos).or_default();

        if vis < THRESHOLD || *pos_count > target_count * 9 {
            miasma_sprite.despawn = true;
        } else {
            *pos_count += 1;
        }
    }
    // Limit the number of cells to check to 8x8 around the player
    const MAX_RADIUS: i64 = 8;
    let min_x = (player_bpos.x - MAX_RADIUS).max(0) as usize;
    let max_x = (player_bpos.x + MAX_RADIUS).min(board_data.map_size.0 as i64 - 1) as usize;
    let min_y = (player_bpos.y - MAX_RADIUS).max(0) as usize;
    let max_y = (player_bpos.y + MAX_RADIUS).min(board_data.map_size.1 as i64 - 1) as usize;
    let z = player_bpos.z as usize;

    for (bp, vis) in vf
        .visibility_field
        .slice(s![min_x..=max_x, min_y..=max_y, z..=z])
        .indexed_iter()
    {
        let bp = (bp.0 + min_x, bp.1 + min_y, bp.2 + z);
        let collision = &board_data.collision_field[bp];
        if !collision.player_free && !collision.see_through {
            continue; // Skip only full walls, allow half-walls for miasma sprites
        }
        let bpos = BoardPosition::from_ndidx(bp);
        let player_dst2 = player_pos.distance2(&bpos.to_position_center());
        let vis = vis + DIST_FACTOR / player_dst2;
        if vis < THRESHOLD * 2.0 {
            continue;
        }
        let target_count = ((board_data.miasma.pressure_field[bpos.ndidx()] / 1.1 + 0.1).min(1.0)
            * MIASMA_TARGET_SPRITE_COUNT as f32) as usize;

        let pos9_count = bpos
            .iter_xy_neighbors(1, board_data.map_size)
            .map(|bpos| count.get(&bpos).copied().unwrap_or_default())
            .sum::<usize>();

        let pos_count = count.entry(bpos.clone()).or_default();

        if pos9_count < target_count * 9 {
            // Spawn miasma if too low
            let scale = rng.random_range(0.15..1.0_f32).sqrt() * 1.8;
            let pos = bpos
                .to_position_center()
                .with_global_z(0.00037 * rng.random_range(0.99..1.01))
                .with_random(0.5);

            commands
                .spawn(Sprite {
                    image: handles.images.miasma.clone(),
                    color: Color::linear_rgba(1.0, 1.0, 1.0, 0.0),
                    anchor: Anchor::Custom(handles.anchors.grid1x1),
                    ..default()
                })
                .insert(MiasmaSprite {
                    base_position: pos,
                    radius: rng.random_range(0.15..0.45), // Small radius
                    angular_speed: rng.random_range(0.05..0.5), // Slow speed
                    phase: rng.random_range(0.0..std::f32::consts::TAU), // Random initial angle. TAU is 2*PI
                    noise_offset_x: rng.random_range(0.0..1000.0),       // Large, distinct offsets
                    noise_offset_y: rng.random_range(0.0..1000.0),
                    visibility: (rng.random_range(0.9..1.0_f32) / scale / 1.3)
                        .powi(2)
                        .clamp(0.3, 2.0),
                    time_alive: 0.0,
                    despawn: false,
                    life: 1.0 + rng.random_range(0.0..0.5),
                    vel_speed: rng.random_range(0.2..1.0_f32).powi(2),
                    direction: board_data.miasma.velocity_field[bpos.ndidx()],
                })
                .insert(Transform::from_scale(Vec3::new(scale, scale, 1.0)))
                .insert(SpriteType::Miasma)
                .insert(pos)
                .insert(GameSprite);
            *pos_count += 1;
        }
    }
    measure.end_ms();
}

fn animate_miasma_sprites(
    time: Res<Time>,
    board_data: Res<BoardData>,
    noise_table: Res<uncore::noise::PerlinNoise>,
    mut query: Query<(&mut Position, &mut MiasmaSprite)>,
) {
    let measure = metrics::ANIMATE_MIASMA.time_measure();

    let dt = time.delta_secs();
    const MOVEMENT_FACTOR: f32 = 1.01;
    for (mut pos, mut miasma_sprite) in query.iter_mut() {
        miasma_sprite.time_alive += dt;
        // 1. Circular Motion:
        let angle = miasma_sprite.angular_speed * miasma_sprite.time_alive + miasma_sprite.phase;
        let circular_x = miasma_sprite.radius * angle.cos();
        let circular_y = miasma_sprite.radius * angle.sin();

        // 2. Perlin Noise Offset using precomputed values:
        let noise_x = noise_table.get(
            miasma_sprite.noise_offset_x + miasma_sprite.time_alive * 0.2,
            miasma_sprite.noise_offset_y,
        );
        let noise_y = noise_table.get(
            miasma_sprite.noise_offset_x,
            miasma_sprite.noise_offset_y + miasma_sprite.time_alive * 0.2,
        );

        // 3. Combine and Update Position:
        pos.x = miasma_sprite.base_position.x + (circular_x + noise_x * 0.6) * MOVEMENT_FACTOR; // Scale noise influence
        pos.y = miasma_sprite.base_position.y + (circular_y + noise_y * 0.6) * MOVEMENT_FACTOR;

        // We do *not* modify pos.z or pos.global_z here.  The Z position is set
        // during initialization and should remain constant.
        let bpos = pos.to_board_position();
        let mut total_vel = Vec2::ZERO;
        let mut total_w = 0.0001;
        for bpos in bpos.iter_xy_neighbors(1, board_data.map_size) {
            if !board_data.collision_field[bpos.ndidx()].player_free {
                continue;
            }
            let w = (bpos.to_position().distance2(&pos) + 0.1).recip();
            let vel = board_data.miasma.velocity_field[bpos.ndidx()];
            total_vel += vel * w;
            total_w += w;
        }
        let vel = total_vel / total_w;
        const F: f32 = 0.1;
        miasma_sprite.direction /= 1.01;
        miasma_sprite.direction = miasma_sprite.direction * (1.0 - F) + vel * F;

        let vel = miasma_sprite.direction;

        let vel_len = (vel.length() + 0.00001) * 5.0;
        let vel = vel * (vel_len.sqrt() / vel_len);
        const SPEED: f32 = 10.9;
        miasma_sprite.base_position.x += vel.x * dt * SPEED * miasma_sprite.vel_speed;
        miasma_sprite.base_position.y += vel.y * dt * SPEED * miasma_sprite.vel_speed;
        let bpos = miasma_sprite.base_position.to_board_position();
        if !board_data
            .collision_field
            .get(bpos.ndidx())
            .map(|collision| collision.player_free)
            .unwrap_or_default()
        {
            let oc_pos = bpos.to_position_center();
            let delta = miasma_sprite.base_position.delta(oc_pos);
            let new_pos = delta.normalized().add_to_position(&oc_pos);
            miasma_sprite.base_position = new_pos;
        }
    }
    measure.end_ms();
}

fn update_miasma(
    mut board_data: ResMut<BoardData>,
    miasma_config: Res<MiasmaConfig>,
    time: Res<Time>,
    roomdb: Res<RoomDB>,
    gc: Res<GameConfig>,
    qp: Query<(&Position, &PlayerSprite)>,
    ghost_query: Query<&GhostSprite>,
    mut bdr_events: EventReader<BoardDataToRebuild>,
    mut room_present: Local<Array3<bool>>,
) {
    let measure = metrics::UPDATE_MIASMA.time_measure();

    // A full board (re)load can change the room layout without changing the map
    // dimensions, so we must not rely on the dimension check alone.
    let board_reloaded = bdr_events
        .read()
        .any(|ev| ev.initialize || ev.collision || ev.lighting);

    let mut rng = random_seed::rng();
    let mut arr = [0u8; 97];
    rng.fill(&mut arr);

    let dt = time.delta_secs();
    let ghosts_remain = !ghost_query.is_empty();
    let diffusion_rate = if ghosts_remain {
        miasma_config.diffusion_rate
    } else {
        miasma_config.diffusion_rate * 20.0
    };
    const EXCHANGE_VEL_SCALE: f32 = 2.0;
    let mut pressure_changes = Array3::from_elem(board_data.map_size, 0.0);
    let mut velocity_changes = Array3::from_elem(board_data.map_size, Vec2::ZERO);

    if board_reloaded || room_present.dim() != board_data.map_size {
        *room_present = Array3::from_elem(board_data.map_size, false);
        for bpos in roomdb.room_tiles.keys() {
            let p = bpos.ndidx();
            if p.0 < board_data.map_size.0
                && p.1 < board_data.map_size.1
                && p.2 < board_data.map_size.2
            {
                room_present[p] = true;
            }
        }
    }

    // Find the active player's position
    let Some(player_pos) = qp.iter().find_map(|(pos, player)| {
        if player.id == gc.player_id {
            Some(*pos)
        } else {
            None
        }
    }) else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    // Iterate through chunks
    let chunks = ChunkIterator::new(board_data.map_size)
        .filter(|chunk| {
            let dist = player_bpos.distance_to_chunk(chunk);
            let r = rng.random_range(1.0..3.0);
            dist < (r * r * r) as i64
        })
        .take(8)
        .collect::<Vec<_>>();
    // if rng.random_range(0..256) == 0 {
    //     dbg!(
    //         chunks.len(),
    //         ChunkIterator::new(board_data.map_size).count()
    //     );
    //     dbg!(
    //         chunks
    //             .iter()
    //             .map(|chunk| CellIterator::new(chunk).count())
    //             .sum::<usize>()
    //     );
    //     let mut distances = chunks
    //         .iter()
    //         .map(|chunk| player_bpos.distance_to_chunk(chunk))
    //         .collect::<Vec<_>>();
    //     distances.sort();
    //     warn!("Chunk distances: {:?}", distances);
    // }
    for chunk in &chunks {
        // Iterate through all cells in the pressure field within the chunk.
        for p in CellIterator::new(chunk) {
            // Check for walls and closed doors (collision)
            // Allow miasma to spread through half-walls (like repellent particles)
            let collision = &board_data.collision_field[p];
            if !collision.player_free && !collision.see_through {
                continue; // Skip full walls that block both movement and sight
            }
            let p1 = board_data.miasma.pressure_field[p];
            let bpos = BoardPosition::from_ndidx(p);
            let is_room = room_present[p];
            // let player_presence =
            //     (256 / (1 + bpos.distance_taxicab(&player_bpos).clamp(0, 64))).clamp(0, 255) as u8;
            // arr_j += 1;
            // arr_j %= arr.len();
            // if arr[arr_j] > player_presence {
            //     continue;
            // }

            // Process each neighbor (up, down, left, right):
            let mut neighbors = vec![bpos.top(), bpos.bottom(), bpos.left(), bpos.right()];

            // Add stair connections for very strong miasma transmission
            let cp = &board_data.collision_field[p];
            if cp.stair_offset != 0 {
                let stair_target_z = bpos.z + cp.stair_offset as i64;
                if stair_target_z >= 0 && stair_target_z < board_data.map_size.2 as i64 {
                    let stair_neighbor = BoardPosition {
                        x: bpos.x,
                        y: bpos.y,
                        z: stair_target_z,
                    };
                    // Add stair neighbor - we'll handle the extra strength in the exchange calculation
                    neighbors.push(stair_neighbor);
                }
            }

            let neighbors = neighbors
                .into_iter()
                .filter(|nb_pos| {
                    let n_idx = nb_pos.ndidx();
                    board_data
                        .collision_field
                        .get(n_idx)
                        .map(|collision| {
                            // Allow miasma to spread through half-walls (like repellent particles)
                            collision.player_free || collision.see_through
                        })
                        .unwrap_or(true)
                })
                .collect::<Vec<_>>();
            let nb_len = neighbors.len() as f32 + 0.01;
            let mut total_v = Vec2::ZERO;
            for neighbor_pos in neighbors {
                if board_data
                    .collision_field
                    .get(neighbor_pos.ndidx())
                    .is_none()
                {
                    continue;
                }
                let np = neighbor_pos.ndidx();
                // Get the neighbor's pressure (treat out-of-bounds as 0.0)
                let mut p2 = board_data
                    .miasma
                    .pressure_field
                    .get(np)
                    .copied()
                    .unwrap_or(0.0);
                let mut v2 = board_data
                    .miasma
                    .velocity_field
                    .get(np)
                    .copied()
                    .unwrap_or(Vec2::ZERO);
                let is_room_nb = room_present[np];
                if !is_room_nb {
                    // Consider outside to be zero pressure and velocity.
                    p2 = 0.0;
                    v2 = Vec2::ZERO;
                }

                total_v += v2;
                // Calculate pressure difference and exchange amount.
                let delta_pressure = p1 - p2;
                let max_exchange_outwards = p1.max(-p2) / 4.0; // Max positive exchange
                let max_exchange_inwards = (-p2).min(p1) / 4.0; // Max negative exchange

                let mut exchange = delta_pressure * diffusion_rate * dt / nb_len;

                // Check if this is a stair connection for super strong miasma flow
                let is_stair_connection = neighbor_pos.z != bpos.z;
                if is_stair_connection {
                    // Miasma flows extremely strongly through stairs - like air
                    // Use 100x stronger diffusion for stairs (increased from 50x)
                    exchange = delta_pressure * diffusion_rate * dt * 100.0 / nb_len;
                } else if !is_room {
                    // Diffuse slower outside of rooms (only for non-stair connections)
                    exchange /= 10.0;
                }

                // --- Biased Diffusion ---
                let velocity = *board_data
                    .miasma
                    .velocity_field
                    .get(p)
                    .unwrap_or(&Vec2::ZERO);
                // Skip velocity adjustments for stair connections since they're vertical
                if !is_stair_connection {
                    // Adjust exchange based on velocity components
                    if neighbor_pos == bpos.top() {
                        exchange -= velocity.y * EXCHANGE_VEL_SCALE;
                    } else if neighbor_pos == bpos.bottom() {
                        exchange += velocity.y * EXCHANGE_VEL_SCALE;
                    } else if neighbor_pos == bpos.left() {
                        exchange -= velocity.x * EXCHANGE_VEL_SCALE;
                    } else if neighbor_pos == bpos.right() {
                        exchange += velocity.x * EXCHANGE_VEL_SCALE;
                    }
                }

                exchange = exchange.clamp(max_exchange_inwards, max_exchange_outwards);

                pressure_changes[p] -= exchange;
                pressure_changes[np] += exchange;
            }
            velocity_changes[p] = total_v / nb_len;
        }
    }

    // Average velocities over space
    for chunk in &chunks {
        // Iterate through all cells in the pressure field within the chunk.
        for p in CellIterator::new(chunk) {
            let vel = velocity_changes[p];
            let Some(entry) = board_data.miasma.velocity_field.get_mut(p) else {
                continue;
            };
            let f = 0.0001;
            *entry = *entry * (1.0 - f) + vel * f;

            // let is_room = room_present[p] && board_data.collision_field[p].player_free;
            // if !is_room {
            //     // Slow particles that aren't in a room.
            //     *entry /= 1.00001;
            // }
        }
    }
    for chunk in &chunks {
        // Iterate through all cells in the pressure field within the chunk.
        for p in CellIterator::new(chunk) {
            let delta = pressure_changes[p];
            let collision = &board_data.collision_field[p];
            let is_room = room_present[p] && (collision.player_free || collision.see_through);

            let Some(entry) = board_data.miasma.pressure_field.get_mut(p) else {
                continue;
            };
            *entry += delta;
            if !is_room {
                // Evaporate miasma fast when outside.
                *entry /= 1.00001;
            }
            if !ghosts_remain {
                // Once every ghost is expelled, evaporate the miasma.
                *entry /= 1.001;
            }
        }
    }

    // --- 2. Velocity Calculation and Inertia ---
    let mut new_velocities = board_data.miasma.velocity_field.clone();
    for chunk in &chunks {
        for p in CellIterator::new(chunk) {
            let p_center = board_data.miasma.pressure_field[p];
            let bpos = BoardPosition::from_ndidx(p);
            let is_room = room_present[p];
            if !is_room {
                // Don't compute velocity outside of rooms.
                // Slow particles that aren't in a room.
                new_velocities[p] /= 1.00001;
                continue;
            }
            // let player_presence = (256
            //     / (1 + (bpos.distance_taxicab(&player_bpos) / 8).clamp(0, 64)))
            // .clamp(0, 255) as u8;
            // arr_j += 1;
            // arr_j %= arr.len();
            // if arr[arr_j] > player_presence {
            //     continue;
            // }

            let get_pressure = |pos: &BoardPosition| -> f32 {
                let gp = pos.ndidx();
                let is_room = room_present[gp];
                if !is_room {
                    // Consider outside to be zero pressure always.
                    return 0.0;
                }
                let collision = &board_data.collision_field[gp];
                if collision.player_free || collision.see_through {
                    // Allow pressure reading from half-walls (like repellent particles)
                    board_data.miasma.pressure_field[gp]
                } else {
                    p_center
                }
            };

            let p_left = get_pressure(&bpos.left());
            let p_right = get_pressure(&bpos.right());
            let p_top = get_pressure(&bpos.top());
            let p_bottom = get_pressure(&bpos.bottom());

            let calculated_velocity = Vec2::new(
                (p_left - p_right) * miasma_config.velocity_scale,
                (p_top - p_bottom) * miasma_config.velocity_scale,
            );
            let calc_vel_len = calculated_velocity.length() + 0.000001;
            let adjusted_vel = calc_vel_len.cbrt().min(1.0);
            let calculated_velocity = calculated_velocity * (adjusted_vel / calc_vel_len); // .min(calculated_velocity);
            let previous_velocity = board_data.miasma.velocity_field[p];

            // FIXME: This should be proportional change of `dt`
            let mut new_velocity = (previous_velocity * miasma_config.inertia_factor
                + calculated_velocity)
                / (1.0 + miasma_config.inertia_factor + miasma_config.friction);

            // Take walls into account.
            const WALL_REPEL_SPEED: f32 = 0.00;
            let old_speed = new_velocity.length();
            if new_velocity.x > -WALL_REPEL_SPEED
                && !board_data
                    .collision_field
                    .get(bpos.right().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.x = -WALL_REPEL_SPEED;
            }
            if new_velocity.x < WALL_REPEL_SPEED
                && !board_data
                    .collision_field
                    .get(bpos.left().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.x = WALL_REPEL_SPEED;
            }
            if new_velocity.y < WALL_REPEL_SPEED
                && !board_data
                    .collision_field
                    .get(bpos.top().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.y = WALL_REPEL_SPEED;
            }
            if new_velocity.y > -WALL_REPEL_SPEED
                && !board_data
                    .collision_field
                    .get(bpos.bottom().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.y = -WALL_REPEL_SPEED;
            }
            new_velocity = new_velocity.normalize_or_zero() * old_speed;
            new_velocities[p] = new_velocity; // Store calculated velocity
        }
    }

    // --- 3. Apply New Velocities ---
    board_data.miasma.velocity_field = new_velocities;

    measure.end_ms();
}

// ---------------------------------------------------------------------------
// Miasma hazards
//
// Heavily pressured miasma occasionally condenses into a drifting hazard that
// homes in on a visible player and burns them. Ported in substance from
// upstream's `environmental-tension-miasma-hazards` branch.
// ---------------------------------------------------------------------------

/// Miasma pressure a tile must exceed before it can condense a hazard.
pub const HAZARD_PRESSURE_THRESHOLD: f32 = 100.0;

/// At this pressure, a tile condenses a hazard with probability
/// `HAZARD_MAX_CHANCE_PER_SEC` per second. Below the threshold the rate falls
/// off linearly.
const HAZARD_PRESSURE_SCALE: f32 = 20000.0;
const HAZARD_MAX_CHANCE_PER_SEC: f32 = 0.05;

/// Hard cap on simultaneous hazards, so a pathological board cannot flood the
/// screen.
const HAZARD_MAX_COUNT: usize = 10;

/// Seconds a hazard lives before it dissipates.
pub const HAZARD_LIFETIME_SECS: f32 = 10.0;

/// Acceleration toward the targeted player, in tiles/s².
const HAZARD_ACCEL: f32 = 10.0;

/// Terminal drift speed, in tiles/s.
const HAZARD_MAX_SPEED: f32 = 2.0;

/// Contact radius for damage, in tiles.
const HAZARD_CONTACT_RADIUS: f32 = 0.5;

/// Health drained per second while a hazard is in contact.
const HAZARD_DAMAGE_PER_SEC: f32 = 50.0;

/// Sprite scale and peak opacity of a hazard, and its tint.
const HAZARD_SPRITE_SCALE: f32 = 0.55;
const HAZARD_MAX_ALPHA: f32 = 0.85;
const HAZARD_TINT: Color = Color::srgb(1.0, 0.35, 0.2);

/// Per-tile probability per second of condensing a hazard from pressure.
///
/// Exposed as a pure function so the falloff can be unit-tested.
pub fn hazard_spawn_chance_per_sec(pressure: f32) -> f32 {
    if pressure <= HAZARD_PRESSURE_THRESHOLD {
        return 0.0;
    }
    ((pressure - HAZARD_PRESSURE_THRESHOLD) / HAZARD_PRESSURE_SCALE)
        .clamp(0.0, HAZARD_MAX_CHANCE_PER_SEC)
}

/// Opacity of a hazard at a given age, fading in and out over its lifetime.
///
/// Hazards should condense and dissipate rather than pop, both for readability
/// and to avoid a sudden flash in peripheral vision.
pub fn hazard_opacity(time_alive: f32) -> f32 {
    const FADE: f32 = 0.6;
    if time_alive <= 0.0 || time_alive >= HAZARD_LIFETIME_SECS {
        return 0.0;
    }
    (time_alive / FADE)
        .min((HAZARD_LIFETIME_SECS - time_alive) / FADE)
        .clamp(0.0, 1.0)
}

/// Condenses hazards out of heavily pressured tiles in the ghost's room.
fn spawn_miasma_hazards(
    time: Res<Time>,
    mut board_data: ResMut<BoardData>,
    ghosts: Query<&Position>,
    roomdb: Res<RoomDB>,
    handles: Res<GameAssets>,
    q_hazards: Query<&MiasmaHazard>,
    mut commands: Commands,
) {
    // Hazards key off the room the ghost is haunting: the pressure build-up that
    // makes a room dangerous should coincide with where the threat is.
    let Some(ghost_pos) = ghosts.iter().next() else {
        return;
    };
    let Some(room_id) = roomdb.room_tiles.get(&ghost_pos.to_board_position()) else {
        return;
    };
    if q_hazards.iter().count() >= HAZARD_MAX_COUNT {
        return;
    }

    let dt = time.delta_secs();
    let mut rng = random_seed::rng();

    for (bpos, r_id) in roomdb.room_tiles.iter() {
        if r_id != room_id {
            continue;
        }
        let idx = bpos.ndidx();
        let pressure = board_data.miasma.pressure_field[idx];
        let chance = hazard_spawn_chance_per_sec(pressure);
        if chance <= 0.0 || !rng.random_bool((chance * dt) as f64) {
            continue;
        }

        // Condensing consumes the pressure that produced it, so a room has to
        // keep churning rather than emitting hazards forever.
        board_data.miasma.pressure_field[idx] = (pressure - HAZARD_PRESSURE_THRESHOLD).max(0.0);

        let spawn = bpos.to_position();
        let pos = Position {
            x: spawn.x + rng.random_range(-0.4..0.4),
            y: spawn.y + rng.random_range(-0.4..0.4),
            z: spawn.z + 0.45,
            global_z: 0.0,
        };

        // Reuse the miasma puff art: a hazard is condensed miasma, so sharing
        // the silhouette keeps the visual language consistent. The hot tint and
        // the fade come from MapColor, which the lighting field also reads, so a
        // hazard darkens with the room instead of glowing through it.
        commands
            .spawn(Sprite {
                image: handles.images.miasma.clone(),
                color: Color::NONE,
                anchor: Anchor::Custom(handles.anchors.grid1x1),
                ..default()
            })
            .insert(Transform::from_scale(Vec3::splat(HAZARD_SPRITE_SCALE)))
            .insert(MapColor {
                color: HAZARD_TINT.with_alpha(0.0),
            })
            .insert(SpriteType::Other)
            .insert(pos)
            .insert(GameSprite)
            .insert(MiasmaHazard::default());
    }
}

/// Drifts hazards toward the nearest visible player on the same floor, bouncing
/// off walls.
fn update_miasma_hazards(
    time: Res<Time>,
    mut commands: Commands,
    mut set: ParamSet<(
        Query<(Entity, &mut Position, &mut MiasmaHazard), Without<PlayerSprite>>,
        Query<&Position, With<PlayerSprite>>,
    )>,
    board_data: Res<BoardData>,
) {
    let dt = time.delta_secs();

    // Collected up front: the ParamSet cannot be borrowed mutably for hazards
    // while the player half is still borrowed.
    let player_positions: Vec<Position> = set.p1().iter().copied().collect();

    for (entity, mut pos, mut hazard) in set.p0().iter_mut() {
        hazard.time_alive += dt;
        if hazard.time_alive >= HAZARD_LIFETIME_SECS {
            commands.entity(entity).despawn();
            continue;
        }

        // Home in on the nearest visible player on this floor.
        let mut nearest: Option<Vec2> = None;
        let mut min_dist2 = f32::MAX;
        for player_pos in player_positions.iter() {
            if player_pos.z.round() as i64 != pos.z.round() as i64 {
                continue;
            }
            let dist2 = player_pos.distance2(&pos);
            if dist2 < min_dist2 {
                min_dist2 = dist2;
                nearest = Some(Vec2::new(player_pos.x, player_pos.y));
            }
        }
        if let Some(target) = nearest {
            let dir = (target - Vec2::new(pos.x, pos.y)).normalize_or_zero();
            hazard.velocity += dir * HAZARD_ACCEL * dt;
        }
        hazard.velocity = hazard.velocity.clamp_length_max(HAZARD_MAX_SPEED);

        let mut next = *pos;
        next.x += hazard.velocity.x * dt;
        next.y += hazard.velocity.y * dt;

        let next_bpos = next.to_board_position();
        match board_data.collision_field.get(next_bpos.ndidx()) {
            Some(collision) if !collision.player_free => {
                // Reflect off whichever axis we tried to cross, then nudge clear
                // of the wall so we cannot get stuck inside it.
                let cur_bpos = pos.to_board_position();
                if next_bpos.x != cur_bpos.x {
                    hazard.velocity.x *= -1.0;
                }
                if next_bpos.y != cur_bpos.y {
                    hazard.velocity.y *= -1.0;
                }
                let push = (pos.to_vec3() - next.to_vec3())
                    .truncate()
                    .normalize_or_zero();
                pos.x += push.x * 0.15;
                pos.y += push.y * 0.15;
            }
            _ => {
                pos.x = next.x;
                pos.y = next.y;
            }
        }
    }
}

/// Burns players who stay in contact with a hazard. Hidden players are safe.
fn miasma_hazard_damage(
    time: Res<Time>,
    hazards: Query<&Position, With<MiasmaHazard>>,
    mut players: Query<(&Position, &mut PlayerSprite), Without<Hiding>>,
) {
    if hazards.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    let contact2 = HAZARD_CONTACT_RADIUS * HAZARD_CONTACT_RADIUS;

    for (pos, mut player) in &mut players {
        let touching = hazards.iter().any(|hazard_pos| {
            hazard_pos.z.round() as i64 == pos.z.round() as i64
                && pos.distance2(hazard_pos) < contact2
        });
        if touching {
            player.health -= HAZARD_DAMAGE_PER_SEC * dt;
        }
    }
}

/// Syncs hazard sprites to their fade curve. The transform translation is
/// handled by `apply_perspective`, which reads `Position`.
fn animate_miasma_hazards(mut hazards: Query<(&mut MapColor, &MiasmaHazard), With<MiasmaHazard>>) {
    for (mut map_color, hazard) in &mut hazards {
        map_color
            .color
            .set_alpha(hazard_opacity(hazard.time_alive) * HAZARD_MAX_ALPHA);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        initialize_miasma.run_if(on_event::<LevelReadyEvent>),
    );
    app.add_systems(Update, spawn_miasma);
    app.add_systems(
        Update,
        (animate_miasma_sprites, update_miasma).run_if(in_state(AppState::InGame)),
    );
    app.add_systems(
        Update,
        (
            spawn_miasma_hazards,
            update_miasma_hazards,
            miasma_hazard_damage,
        )
            .run_if(in_state(AppState::InGame)),
    );
    app.add_systems(Update, animate_miasma_hazards);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hazard_spawn_chance_is_zero_below_the_threshold() {
        assert_eq!(hazard_spawn_chance_per_sec(0.0), 0.0);
        assert_eq!(hazard_spawn_chance_per_sec(50.0), 0.0);
        assert_eq!(
            hazard_spawn_chance_per_sec(HAZARD_PRESSURE_THRESHOLD),
            0.0,
            "exactly at the threshold nothing should condense"
        );
        assert_eq!(
            hazard_spawn_chance_per_sec(HAZARD_PRESSURE_THRESHOLD - 1.0),
            0.0,
            "just under the threshold nothing should condense"
        );
    }

    #[test]
    fn hazard_spawn_chance_rises_then_saturates() {
        let low = hazard_spawn_chance_per_sec(HAZARD_PRESSURE_THRESHOLD + 1.0);
        let mid = hazard_spawn_chance_per_sec(HAZARD_PRESSURE_THRESHOLD + 10_000.0);
        let high = hazard_spawn_chance_per_sec(1.0e9);

        assert!(low > 0.0);
        assert!(mid > low, "chance must grow with pressure");
        assert_eq!(high, HAZARD_MAX_CHANCE_PER_SEC, "chance must saturate");
        assert!(mid <= HAZARD_MAX_CHANCE_PER_SEC);
    }

    #[test]
    fn hazard_spawn_chance_is_always_a_probability() {
        for pressure in [100.0, 150.0, 1_000.0, 1.0e6, f32::MAX] {
            let chance = hazard_spawn_chance_per_sec(pressure);
            assert!(
                (0.0..=HAZARD_MAX_CHANCE_PER_SEC).contains(&chance),
                "chance {chance} out of range at pressure {pressure}"
            );
        }
    }

    #[test]
    fn hazard_opacity_fades_in_and_out() {
        assert_eq!(hazard_opacity(0.0), 0.0, "must not pop into existence");
        assert_eq!(
            hazard_opacity(HAZARD_LIFETIME_SECS),
            0.0,
            "must dissipate before despawning"
        );
        assert_eq!(
            hazard_opacity(HAZARD_LIFETIME_SECS + 5.0),
            0.0,
            "past its lifetime it is fully faded"
        );

        let mid = hazard_opacity(HAZARD_LIFETIME_SECS / 2.0);
        assert!(mid > 0.9, "peak opacity should reach the top of the curve");
    }

    #[test]
    fn hazard_opacity_is_monotonic_up_then_down() {
        let mut previous = -1.0;
        let mut rising = true;
        let mut t = 0.0;
        while t <= HAZARD_LIFETIME_SECS {
            let alpha = hazard_opacity(t);
            assert!((0.0..=1.0).contains(&alpha), "alpha {alpha} out of range");

            if alpha < previous {
                rising = false;
            } else if rising {
                previous = alpha;
            } else {
                // Once descending it must stay descending.
                assert!(
                    alpha <= previous,
                    "opacity must not rise again after peaking"
                );
                previous = alpha;
            }
            t += 0.1;
        }
    }

    /// A hazard at peak opacity must not exceed the ceiling the animate system
    /// multiplies into `MapColor`.
    #[test]
    fn hazard_peak_alpha_is_within_the_render_ceiling() {
        let peak = hazard_opacity(HAZARD_LIFETIME_SECS / 2.0) * HAZARD_MAX_ALPHA;
        assert!(peak <= 1.0, "hazard alpha {peak} would exceed full opacity");
    }
}
