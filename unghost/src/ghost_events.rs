use bevy::prelude::*;
use rand::Rng;
use uncore::behaviour::{Behavior, component};
use uncore::components::board::position::Position;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::events::sound::SoundEvent;
use uncore::random_seed;
use unstd::board::spritedb::SpriteDB;
use unstd::systemparam::interactivestuff::InteractiveStuff;

#[derive(Debug, Clone)]
pub enum GhostEvent {
    DoorSlam,
    LightFlicker,
}

#[derive(Component)]
struct FlickerTimer(Timer);

pub fn trigger_ghost_events(
    mut commands: Commands,
    q_player: Query<(&Position, &PlayerSprite)>,
    q_ghost: Query<(&GhostSprite, &Position)>,
    // Query for doors, excluding lights
    q_doors: Query<
        (Entity, &Position, &Behavior),
        (
            With<component::Door>,
            Without<component::Light>,
        ),
    >,
    // Query for lights, excluding doors
    mut q_lights: Query<
        (Entity, &Position, &mut Behavior),
        (
            With<component::Light>,
            Without<component::Interactive>,
        ),
    >,
    mut interactive_stuff: InteractiveStuff,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    difficulty: Res<CurrentDifficulty>,
) {
    let mut rng = random_seed::rng();
    let roomdb = interactive_stuff.roomdb.clone();

    // Iterate through players inside the house
    for (player_pos, _player) in q_player.iter().filter(|(pos, _)| {
        let bpos = pos.to_board_position();
        roomdb.room_tiles.contains_key(&bpos)
    }) {
        // Find the ghost
        let Ok((_ghost, ghost_pos)) = q_ghost.single() else {
            return;
        };

        // Calculate distance and event probability
        let distance = player_pos.distance2(ghost_pos);
        let event_probability =
            (10.0 / (distance + 2.0)).sqrt() / 200.0 * difficulty.0.ghost_interaction_frequency;

        // Roll for an event
        if rng.random_range(0.0..1.0) < event_probability {
            // Choose a random event
            let event = match rng.random_range(0..10) {
                0 => GhostEvent::DoorSlam,
                _ => GhostEvent::LightFlicker,
            };
            // warn!("Event: {:?}", event);
            match event {
                GhostEvent::DoorSlam => {
                    // Find doors in the player's room
                    let player_room = roomdb
                        .room_tiles
                        .get(&player_pos.to_board_position())
                        .cloned();
                    let mut doors_in_room = Vec::new();
                    if let Some(player_room) = player_room {
                        for (entity, door_pos, behavior) in q_doors.iter() {
                            if roomdb.room_tiles.get(&door_pos.to_board_position())
                                == Some(&player_room)
                                && !behavior.p.movement.player_collision
                            {
                                // Just put here the open doors as candidates.
                                doors_in_room.push(entity);
                            }
                        }
                    }

                    // If there are doors, slam a random one
                    if !doors_in_room.is_empty() {
                        let door_to_slam = doors_in_room[rng.random_range(0..doors_in_room.len())];

                        // Retrieve the door's Behavior component
                        if let Ok((door_entity, door_position, behavior)) = q_doors.get(door_to_slam) {
                            // Use proper ghost door interaction instead of player interaction system
                            if let Some(alternative_behavior) = find_alternative_door_state(&interactive_stuff.bf, behavior) {
                                let mut door_commands = interactive_stuff.commands.get_entity(door_entity).unwrap();
                                
                                // Update the door's behavior to the alternative state (closed)
                                door_commands.insert(alternative_behavior);
                                
                                // Update the door's visual appearance
                                let cvo = behavior.key_cvo();
                                if let Some(other_tuids) = interactive_stuff.bf.cvo_idx.get(&cvo) {
                                    let tuid = behavior.key_tuid();
                                    for other_tuid in other_tuids {
                                        if *other_tuid != tuid {
                                            if let Some(other_tile) = interactive_stuff.bf.map_tile.get(other_tuid) {
                                                let b = other_tile.bundle.clone();
                                                let mat = interactive_stuff.materials1.get(&b.material).unwrap().clone();
                                                let mat = interactive_stuff.materials1.add(mat);
                                                door_commands.insert(MeshMaterial2d(mat));
                                                break;
                                            }
                                        }
                                    }
                                }

                                // Play door slam sound effect
                                interactive_stuff.sound_events.write(SoundEvent {
                                    sound_file: "sounds/door-close.ogg".to_string(),
                                    volume: 1.0,
                                    position: Some(*door_position),
                                });

                                ev_bdr.write(BoardDataToRebuild {
                                    lighting: true,
                                    collision: true,
                                });
                            }
                        }
                        // warn!("Slamming door: {:?}", door_to_slam);
                    }
                }
                GhostEvent::LightFlicker => {
                    // Find lights in the player's room
                    let player_room = roomdb
                        .room_tiles
                        .get(&player_pos.to_board_position())
                        .cloned();
                    if let Some(player_room) = player_room {
                        let mut flicker = false;
                        for (entity, light_pos, mut behavior) in q_lights.iter_mut() {
                            if behavior.can_emit_light()
                                && roomdb.room_tiles.get(&light_pos.to_board_position())
                                    == Some(&player_room)
                            {
                                // Toggle the light's state using the public method
                                behavior.p.light.flickering = true;

                                // Add a timer to reset the light after a short duration
                                commands
                                    .entity(entity)
                                    .insert(FlickerTimer(Timer::from_seconds(
                                        0.5,
                                        TimerMode::Once,
                                    )));
                                // warn!("Flickering light: {:?}", entity);
                                flicker = true;
                            }
                        }
                        if flicker {
                            ev_bdr.write(BoardDataToRebuild {
                                lighting: true,
                                collision: true,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn update_flicker_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut q_lights: Query<(Entity, &mut FlickerTimer, &mut Behavior)>,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
) {
    for (entity, mut flicker_timer, mut behavior) in q_lights.iter_mut() {
        flicker_timer.0.tick(time.delta());
        if flicker_timer.0.finished() {
            // Reset the light to its original state using the public method
            behavior.p.light.flickering = false;
            commands.entity(entity).remove::<FlickerTimer>();
            ev_bdr.write(BoardDataToRebuild {
                lighting: true,
                collision: true,
            });
        }
    }
}

/// Finds an alternative door state (Open -> Closed or Closed -> Open) using the SpriteDB
/// This is a ghost-specific door interaction that doesn't use the player interaction system
fn find_alternative_door_state(sprite_db: &SpriteDB, current_behavior: &Behavior) -> Option<Behavior> {
    let cvo = current_behavior.key_cvo();
    
    // Get all tiles with the same class, variant, and orientation
    if let Some(other_tuids) = sprite_db.cvo_idx.get(&cvo) {
        let current_tuid = current_behavior.key_tuid();
        
        for other_tuid in other_tuids {
            if *other_tuid != current_tuid {
                if let Some(other_tile) = sprite_db.map_tile.get(other_tuid) {
                    let mut alternative_behavior = other_tile.behavior.clone();
                    // Preserve the flip state from the original door
                    alternative_behavior.flip(current_behavior.p.flip);
                    return Some(alternative_behavior);
                }
            }
        }
    }
    
    None
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, (trigger_ghost_events, update_flicker_timers));
}
