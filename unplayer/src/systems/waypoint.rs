use bevy::{prelude::*, window::PrimaryWindow};
use uncore::{
    behaviour::{Behaviour, component::Interactive, component::Stairs},
    components::{
        board::{PERSPECTIVE_X, PERSPECTIVE_Y, PERSPECTIVE_Z, position::Position},
        game::{GCameraArena, GameSprite},
        player_sprite::PlayerSprite,
        waypoint::{Waypoint, WaypointOwner, WaypointQueue, WaypointType},
    },
    events::roomchanged::{InteractionExecutionType, RoomChangedEvent},
    resources::{
        board_data::BoardData, mouse_visibility::MouseVisibility, player_input::PlayerInput,
        visibility_data::VisibilityData,
    },
};
use unstd::systemparam::interactivestuff::InteractiveStuff;

use super::pathfinding::{detect_stair_area, find_path, find_path_to_interactive};

/// System that creates waypoint entities when the player clicks.
/// Handles both interactive objects (via picking) and ground clicks (via raw mouse input).
/// Only allows clicks on interactive entities that are on the same floor as the player.
pub fn waypoint_creation_system(
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GCameraArena>>,
    q_player: Query<(Entity, &Position), With<PlayerSprite>>,
    mut q_player_queue: Query<&mut WaypointQueue, With<PlayerSprite>>,
    q_existing_waypoints: Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    q_interactives: Query<(
        Entity,
        &Position,
        &Interactive,
        &Behaviour,
        Option<&uncore::behaviour::component::RoomState>,
    )>,
    q_stairs: Query<(Entity, &Position, &Stairs, &Behaviour)>,
    mut click_events: EventReader<bevy::picking::events::Pointer<bevy::picking::events::Click>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_visibility: Res<MouseVisibility>,
    board_data: Res<BoardData>,
    visibility_data: Res<VisibilityData>,
) {
    // Only process clicks when mouse is visible
    if !mouse_visibility.is_visible {
        return;
    }

    let Ok((player_entity, player_pos)) = q_player.single() else {
        return;
    };

    let Ok(mut waypoint_queue) = q_player_queue.single_mut() else {
        return;
    };

    // Find the active player's position and floor
    let player_floor = player_pos.z.round() as i32;

    // Track if any interactive object was clicked via picking events
    let mut interactive_clicked = false;

    // First, handle picking events for interactive objects
    for click_event in click_events.read() {
        // Only handle left clicks
        if click_event.button != PointerButton::Primary {
            continue;
        }

        // Check if clicked on an interactive entity
        if let Ok((interactive_entity, interactive_pos, _interactive, _behaviour, _room_state)) =
            q_interactives.get(click_event.target)
        {
            let interactive_floor = interactive_pos.z.round() as i32;

            // Only allow clicks on interactives that are on the same floor as the player
            if interactive_floor != player_floor {
                debug!(
                    "waypoint_creation_system: Ignoring click on interactive entity {:?} - player on floor {}, interactive on floor {}",
                    interactive_entity, player_floor, interactive_floor
                );
                continue; // Skip this click
            }

            debug!(
                "waypoint_creation_system: Creating waypoint to interactive entity {:?} on floor {}",
                interactive_entity, interactive_floor
            );

            // Clear existing waypoints when creating new ones
            clear_player_waypoints(
                &mut commands,
                &q_existing_waypoints,
                player_entity,
                &mut waypoint_queue,
            );

            // Check if we're close enough to interact directly
            const INTERACTION_DISTANCE: f32 = 1.2;
            let distance = player_pos.distance(interactive_pos);

            if distance <= INTERACTION_DISTANCE {
                // Close enough - create direct interaction waypoint
                create_interaction_waypoint(
                    &mut commands,
                    player_entity,
                    *interactive_pos,
                    interactive_entity,
                    &mut waypoint_queue,
                );
            } else {
                // Too far - use pathfinding to get close, then create interaction waypoint
                create_pathfinding_waypoints_to_interaction(
                    &mut commands,
                    &q_existing_waypoints,
                    player_entity,
                    *player_pos,
                    *interactive_pos,
                    interactive_entity,
                    &mut waypoint_queue,
                    &board_data,
                    &visibility_data,
                );
            }
            interactive_clicked = true;
        }
    }

    // If no interactive was clicked, check for ground clicks via raw mouse input
    if !interactive_clicked && mouse.just_pressed(MouseButton::Left) {
        let Ok(window) = q_window.single() else {
            return;
        };
        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };
        let Ok((camera, camera_transform)) = q_camera.single() else {
            return;
        };

        // Convert cursor position to world coordinates
        if let Some(target) =
            screen_to_world_coords(cursor_pos, player_pos.z, camera, camera_transform)
        {
            debug!("Ground click detected at {:?}", target);

            // First check if the click is in a stairs area
            if let Some((
                _stair_entity,
                _stair_pos,
                _stair_component,
                _behaviour,
                start_waypoint,
                end_waypoint,
            )) = detect_stair_area(target, &q_stairs)
            {
                debug!(
                    "Stair area detected! Creating waypoints from {:?} to {:?}",
                    start_waypoint, end_waypoint
                );
                // Create stair traversal waypoints
                create_stair_waypoints(
                    &mut commands,
                    &q_existing_waypoints,
                    player_entity,
                    start_waypoint,
                    end_waypoint,
                    &mut waypoint_queue,
                );
            } else {
                // Use pathfinding to create a sequence of waypoints
                create_pathfinding_waypoints(
                    &mut commands,
                    &q_existing_waypoints,
                    player_entity,
                    *player_pos,
                    target,
                    &mut waypoint_queue,
                    &board_data,
                    &visibility_data,
                );
            }
        }
    }
}

/// System that makes the player follow waypoints.
/// Replaces the old click-to-move update system.
pub fn waypoint_following_system(
    mut commands: Commands,
    q_player: Query<(Entity, &Position, &WaypointQueue), With<PlayerSprite>>,
    q_waypoints: Query<(&Position, &Waypoint), (With<WaypointOwner>, Without<PlayerSprite>)>,
    q_interactives: Query<(
        Entity,
        &Position,
        &Interactive,
        &Behaviour,
        Option<&uncore::behaviour::component::RoomState>,
    )>,
    mut player_input: ResMut<PlayerInput>,
    mut interactive_stuff: InteractiveStuff,
    mut ev_room: EventWriter<RoomChangedEvent>,
) {
    for (player_entity, player_pos, waypoint_queue) in q_player.iter() {
        if let Some(current_waypoint_entity) = waypoint_queue.next() {
            if let Ok((waypoint_pos, waypoint)) = q_waypoints.get(current_waypoint_entity) {
                let current = Vec2::new(player_pos.x, player_pos.y);
                let target = Vec2::new(waypoint_pos.x, waypoint_pos.y);
                let to_target = target - current;

                const ARRIVAL_THRESHOLD: f32 = 0.1;
                const INTERACTION_DISTANCE: f32 = 1.2;

                // Check if we should handle the waypoint action
                let should_complete_waypoint = match &waypoint.waypoint_type {
                    WaypointType::MoveTo => {
                        // For movement waypoints, complete when we reach the position
                        to_target.length_squared() <= ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD
                    }
                    WaypointType::Interact(interaction_target) => {
                        // For interaction waypoints, try to interact as soon as we're close enough
                        if let Ok((_, interactive_pos, interactive, behaviour, room_state)) =
                            q_interactives.get(*interaction_target)
                        {
                            let distance = player_pos.distance(interactive_pos);
                            if distance <= INTERACTION_DISTANCE {
                                // Execute the interaction
                                if interactive_stuff.execute_interaction(
                                    *interaction_target,
                                    interactive_pos,
                                    Some(interactive),
                                    behaviour,
                                    room_state,
                                    InteractionExecutionType::ChangeState,
                                ) {
                                    ev_room.write(RoomChangedEvent::default());
                                }
                                true // Complete the waypoint after interaction
                            } else {
                                // Still too far, keep moving
                                false
                            }
                        } else {
                            // Target entity no longer exists, complete waypoint
                            true
                        }
                    }
                };

                if should_complete_waypoint {
                    // Complete waypoint and stop moving
                    player_input.movement = Vec2::ZERO;
                    complete_waypoint(&mut commands, player_entity, current_waypoint_entity);
                } else {
                    // Continue moving towards waypoint
                    player_input.movement = to_target.normalize();
                }
            } else {
                // Waypoint entity no longer exists, remove it from queue
                complete_waypoint(&mut commands, player_entity, current_waypoint_entity);
            }
        }
    }
}

/// Helper function to clear all waypoints belonging to a player
fn clear_player_waypoints(
    commands: &mut Commands,
    q_existing_waypoints: &Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    _player_entity: Entity,
    waypoint_queue: &mut WaypointQueue,
) {
    // Despawn all waypoint entities belonging to this player
    for waypoint_entity in &waypoint_queue.0 {
        if q_existing_waypoints.contains(*waypoint_entity) {
            commands.entity(*waypoint_entity).despawn();
        }
    }
    waypoint_queue.clear();
}

/// Helper function to create an interaction waypoint
fn create_interaction_waypoint(
    commands: &mut Commands,
    player_entity: Entity,
    position: Position,
    interaction_target: Entity,
    waypoint_queue: &mut WaypointQueue,
) {
    let waypoint_entity = commands
        .spawn(Sprite {
            color: Color::srgba(1.0, 1.0, 0.0, 0.6), // Yellow for interaction waypoints
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..default()
        })
        .insert(position)
        .insert(GameSprite)
        .insert(Waypoint {
            waypoint_type: WaypointType::Interact(interaction_target),
            order: 0,
        })
        .insert(WaypointOwner(player_entity))
        .id();

    waypoint_queue.push(waypoint_entity);
}

/// Helper function to create a move-to waypoint
pub fn create_move_waypoint(
    commands: &mut Commands,
    player_entity: Entity,
    position: Position,
    waypoint_queue: &mut WaypointQueue,
) {
    let waypoint_entity = commands
        .spawn(Sprite {
            color: Color::srgba(1.0, 0.0, 0.6, 0.8), // Red for move waypoints
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..default()
        })
        .insert(position)
        .insert(GameSprite)
        .insert(Waypoint {
            waypoint_type: WaypointType::MoveTo,
            order: 0,
        })
        .insert(WaypointOwner(player_entity))
        .id();

    waypoint_queue.push(waypoint_entity);
}

/// Helper function to complete a waypoint (remove it and advance queue)
fn complete_waypoint(commands: &mut Commands, _player_entity: Entity, waypoint_entity: Entity) {
    // Despawn the waypoint entity
    commands.entity(waypoint_entity).despawn();

    // Remove from player's queue (will be handled by queue management system)
    // For now we'll update it in the next system run
}

/// System that cleans up waypoint queues by removing despawned waypoint entities
pub fn waypoint_queue_cleanup_system(
    mut q_player_queue: Query<&mut WaypointQueue, With<PlayerSprite>>,
    q_waypoints: Query<Entity, With<Waypoint>>,
) {
    for mut waypoint_queue in q_player_queue.iter_mut() {
        // Remove any waypoint entities that no longer exist
        waypoint_queue
            .0
            .retain(|&waypoint_entity| q_waypoints.contains(waypoint_entity));
    }
}

/// Converts screen coordinates to world coordinates using the game's isometric projection.
fn screen_to_world_coords(
    screen_pos: Vec2,
    target_z: f32,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Position> {
    // Get the world position on the camera's near plane using Bevy's built-in conversion
    let world_pos_on_near_plane = camera
        .viewport_to_world_2d(camera_transform, screen_pos)
        .ok()?;

    // Calculate the determinant of the 2x2 isometric projection matrix
    let det = PERSPECTIVE_X[0] * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * PERSPECTIVE_X[1];
    if det.abs() < 1e-6 {
        return None; // Matrix is not invertible
    }
    let inv_det = 1.0 / det;

    // Adjust screen coordinates by removing the Z-level contribution
    let b_x = world_pos_on_near_plane.x - target_z * PERSPECTIVE_Z[0];
    let b_y = world_pos_on_near_plane.y - target_z * PERSPECTIVE_Z[1];

    // Apply the inverse transformation matrix to find world X and Y coordinates
    let world_x = inv_det * (b_x * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * b_y);
    let world_y = inv_det * (PERSPECTIVE_X[0] * b_y - b_x * PERSPECTIVE_X[1]);

    Some(Position {
        x: world_x,
        y: world_y,
        z: target_z,
        global_z: 0.0,
    })
}

/// Helper function to create waypoints using pathfinding
fn create_pathfinding_waypoints(
    commands: &mut Commands,
    q_existing_waypoints: &Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    player_entity: Entity,
    start_pos: Position,
    target_pos: Position,
    waypoint_queue: &mut WaypointQueue,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) {
    // Clear existing waypoints first
    clear_player_waypoints(
        commands,
        q_existing_waypoints,
        player_entity,
        waypoint_queue,
    );

    // Use pathfinding to get a sequence of board positions
    let path = find_path(start_pos, target_pos, board_data, visibility_data);

    if path.is_empty() {
        debug!("No path found from {:?} to {:?}", start_pos, target_pos);
        return;
    }

    // Skip the first position (current player position) and create waypoints for the rest
    for (i, board_pos) in path.iter().skip(1).enumerate() {
        let world_pos = board_pos.to_position();

        let waypoint_entity = commands
            .spawn(Sprite {
                color: Color::srgba(1.0, 0.0, 0.6, 0.8), // Red for move waypoints
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            })
            .insert(world_pos)
            .insert(GameSprite)
            .insert(Waypoint {
                waypoint_type: WaypointType::MoveTo,
                order: i as u32,
            })
            .insert(WaypointOwner(player_entity))
            .id();

        waypoint_queue.push(waypoint_entity);
    }

    debug!("Created {} waypoints for pathfinding", path.len() - 1);
}

/// Helper function to create waypoints using pathfinding that end with an interaction
fn create_pathfinding_waypoints_to_interaction(
    commands: &mut Commands,
    q_existing_waypoints: &Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    player_entity: Entity,
    start_pos: Position,
    target_pos: Position,
    interaction_target: Entity,
    waypoint_queue: &mut WaypointQueue,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
) {
    // Clear existing waypoints first
    clear_player_waypoints(
        commands,
        q_existing_waypoints,
        player_entity,
        waypoint_queue,
    );

    // Use pathfinding to get a sequence of board positions (treating target as walkable)
    let path = find_path_to_interactive(start_pos, target_pos, board_data, visibility_data);

    if path.is_empty() {
        debug!("No path found from {:?} to {:?}", start_pos, target_pos);
        // If no path, create direct interaction waypoint as fallback
        create_interaction_waypoint(
            commands,
            player_entity,
            target_pos,
            interaction_target,
            waypoint_queue,
        );
        return;
    }

    // Create movement waypoints for all but the last position
    let path_len = path.len();
    for (i, board_pos) in path.iter().skip(1).enumerate() {
        // If this is the last waypoint, create an interaction waypoint instead
        if i == path_len - 2 {
            // Last iteration (path_len - 1 because we skip(1))
            create_interaction_waypoint(
                commands,
                player_entity,
                target_pos,
                interaction_target,
                waypoint_queue,
            );
        } else {
            // Create regular movement waypoint
            let world_pos = board_pos.to_position();

            let waypoint_entity = commands
                .spawn(Sprite {
                    color: Color::srgba(1.0, 0.0, 0.6, 0.8), // Red for move waypoints
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                })
                .insert(world_pos)
                .insert(GameSprite)
                .insert(Waypoint {
                    waypoint_type: WaypointType::MoveTo,
                    order: i as u32,
                })
                .insert(WaypointOwner(player_entity))
                .id();

            waypoint_queue.push(waypoint_entity);
        }
    }

    debug!(
        "Created {} waypoints for pathfinding to interaction",
        path_len - 1
    );
}

/// Helper function to create waypoints for stair traversal
fn create_stair_waypoints(
    commands: &mut Commands,
    q_existing_waypoints: &Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    player_entity: Entity,
    start_waypoint: Position,
    end_waypoint: Position,
    waypoint_queue: &mut WaypointQueue,
) {
    // Clear existing waypoints first
    clear_player_waypoints(
        commands,
        q_existing_waypoints,
        player_entity,
        waypoint_queue,
    );

    debug!(
        "Creating stair waypoints from {:?} to {:?}",
        start_waypoint, end_waypoint
    );

    // Create start waypoint (on current floor)
    let start_waypoint_entity = commands
        .spawn(Sprite {
            color: Color::srgba(0.0, 1.0, 1.0, 0.8), // Cyan for stair start
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..default()
        })
        .insert(start_waypoint)
        .insert(GameSprite)
        .insert(Waypoint {
            waypoint_type: WaypointType::MoveTo,
            order: 0,
        })
        .insert(WaypointOwner(player_entity))
        .id();

    waypoint_queue.push(start_waypoint_entity);

    // Create end waypoint (on destination floor)
    let end_waypoint_entity = commands
        .spawn(Sprite {
            color: Color::srgba(1.0, 1.0, 0.0, 0.8), // Yellow for stair end
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..default()
        })
        .insert(end_waypoint)
        .insert(GameSprite)
        .insert(Waypoint {
            waypoint_type: WaypointType::MoveTo,
            order: 1,
        })
        .insert(WaypointOwner(player_entity))
        .id();

    waypoint_queue.push(end_waypoint_entity);

    debug!("Created 2 waypoints for stair traversal");
}
