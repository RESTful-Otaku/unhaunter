use super::components::deployedgear::{DeployedGear, DeployedGearData};
use super::components::playergear::PlayerGear;
use crate::gear_stuff::GearStuff;
use crate::gear_usable::GearUsable;
use bevy::audio::SpatialScale;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::components::board::position::Position;
use uncore::components::game_config::GameConfig;
use uncore::components::player_inventory::{Inventory, InventoryNext, InventoryStats};
use uncore::components::player_sprite::PlayerSprite;
use uncore::events::sound::SoundEvent;
use uncore::resources::looking_gear::LookingGear;
use uncore::states::GameState;
use uncore::types::gear::equipmentposition::{EquipmentPosition, Hand};
use unsettings::audio::{AudioSettings, SoundOutput, AudioPositioning};

/// System for updating the internal state of all gear carried by the player.
///
/// This system iterates through the player's gear and calls the `update` method
/// for each piece of gear, allowing gear to update their state based on time,
/// player actions, or environmental conditions.
fn update_playerheld_gear_data(mut q_gear: Query<(&Position, &mut PlayerGear)>, mut gs: GearStuff) {
    for (position, mut playergear) in q_gear.iter_mut() {
        for (gear, epos) in playergear.as_vec_mut().into_iter() {
            gear.update(&mut gs, position, &epos);
        }
    }
}

/// System for updating the internal state of all gear deployed in the environment.
fn update_deployed_gear_data(
    mut q_gear: Query<(&Position, &DeployedGear, &mut DeployedGearData)>,
    mut gs: GearStuff,
) {
    for (position, _deployed_gear, mut gear_data) in q_gear.iter_mut() {
        gear_data
            .gear
            .update(&mut gs, position, &EquipmentPosition::Deployed);
    }
}

/// System for updating the sprites of deployed gear to reflect their internal
/// state.
fn update_deployed_gear_sprites(mut q_gear: Query<(&mut Sprite, &DeployedGearData)>) {
    for (mut sprite, gear_data) in q_gear.iter_mut() {
        let new_index = gear_data.gear.get_sprite_idx() as usize;
        if let Some(texture_atlas) = &mut sprite.texture_atlas {
            if texture_atlas.index != new_index {
                texture_atlas.index = new_index;
            }
        }
    }
}

/// System to handle the SoundEvent, playing the sound with volume adjusted by
/// distance and stereo positioning based on audio positioning mode.
fn sound_playback_system(
    mut sound_events: EventReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    gc: Res<GameConfig>,
    qp: Query<(Entity, &Position, &PlayerSprite)>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    for sound_event in sound_events.read() {
        // Get player position
        let Some((_player_entity, player_position, _)) =
            qp.iter().find(|(_, _, p)| p.id == gc.player_id)
        else {
            return;
        };
        if !player_position.is_finite() {
            warn!("Player position is not finite: {player_position:?}")
        }
        
        // Calculate distance-based volume adjustment
        let dist = sound_event
            .position
            .map(|pos| player_position.distance(&pos))
            .unwrap_or(0.0);
        let mut adjusted_volume = (sound_event.volume * (1.0 + dist * 0.2)).clamp(0.0, 1.0);
        if audio_settings.sound_output == SoundOutput::Mono {
            adjusted_volume /= 1.0 + dist * 0.4;
        }

        // Spawn an AudioBundle with the adjusted volume
        let mut sound = commands.spawn(AudioPlayer::<AudioSource>(
            asset_server.load(sound_event.sound_file.clone()),
        ));
        
        // Apply stereo positioning based on audio positioning mode
        let (spatial_enabled, spatial_transform) = if let Some(position) = sound_event.position {
            match audio_settings.audio_positioning {
                AudioPositioning::ScreenSpace => {
                    // Simple screen-based positioning (left/right only)
                    let mut spos_vec = position.to_screen_coord();
                    spos_vec.z -= 10.0 / audio_settings.sound_output.to_ear_offset();
                    (audio_settings.sound_output != SoundOutput::Mono, spos_vec)
                }
                AudioPositioning::Isometric => {
                    // Isometric positioning - map world position to stereo field
                    let player_screen = player_position.to_screen_coord();
                    let sound_screen = position.to_screen_coord();
                    
                    // Calculate relative position in isometric view
                    let dx = sound_screen.x - player_screen.x;
                    let dy = sound_screen.y - player_screen.y;
                    
                    // Convert to stereo positioning
                    // In isometric view, we map the diagonal world to stereo field
                    // Front-right sounds go to right ear, front-left to left ear
                    let stereo_x = (dx + dy * 0.5) * 0.02; // Scale factor for stereo width
                    let stereo_z = -10.0 / audio_settings.sound_output.to_ear_offset();
                    
                    let spos_vec = Vec3::new(stereo_x, sound_screen.y, stereo_z);
                    (audio_settings.sound_output != SoundOutput::Mono, spos_vec)
                }
                AudioPositioning::CharacterRelative => {
                    // FPS-style positioning - full 360 degree coverage
                    let player_screen = player_position.to_screen_coord();
                    let sound_screen = position.to_screen_coord();
                    
                    // Calculate angle relative to player's facing direction
                    let dx = sound_screen.x - player_screen.x;
                    let dy = sound_screen.y - player_screen.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance > 0.0 {
                        // Calculate angle (0 = front, PI/2 = right, PI = back, 3PI/2 = left)
                        let angle = dy.atan2(dx);
                        
                        // Convert angle to stereo positioning
                        // Front (0°) = center, right (90°) = right ear, back (180°) = center, left (270°) = left ear
                        let stereo_x = angle.sin() * 5.0; // Stereo width
                        let stereo_z = -10.0 / audio_settings.sound_output.to_ear_offset();
                        
                        let spos_vec = Vec3::new(stereo_x, sound_screen.y, stereo_z);
                        (audio_settings.sound_output != SoundOutput::Mono, spos_vec)
                    } else {
                        // Sound is at player position - center it
                        let spos_vec = Vec3::new(0.0, player_screen.y, -10.0 / audio_settings.sound_output.to_ear_offset());
                        (audio_settings.sound_output != SoundOutput::Mono, spos_vec)
                    }
                }
            }
        } else {
            // No position specified - non-spatial sound
            (false, Vec3::ZERO)
        };

        sound.insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(
                adjusted_volume
                    * audio_settings.volume_effects.as_f32()
                    * audio_settings.volume_master.as_f32(),
            ),
            speed: 1.0,
            paused: false,
            spatial: spatial_enabled,
            spatial_scale: Some(SpatialScale::new(0.005)),
            ..default()
        });

        if spatial_enabled {
            sound.insert(Transform::from_translation(spatial_transform));
        }
    }
}

fn keyboard_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    looking_gear: Res<LookingGear>,
    mut gs: GearStuff,
) {
    for (ps, mut playergear) in q_gear.iter_mut() {
        if keyboard_input.just_pressed(ps.controls.cycle) {
            playergear.cycle(&looking_gear.hand());
        }
        if keyboard_input.just_pressed(ps.controls.swap) {
            playergear.swap();
        }
        if keyboard_input.just_released(ps.controls.trigger) {
            playergear.right_hand.set_trigger(&mut gs);
        }
        if keyboard_input.just_released(ps.controls.torch) {
            playergear.left_hand.set_trigger(&mut gs);
        }
    }
}

fn update_gear_ui(
    gc: Res<GameConfig>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    mut qi: Query<(&Inventory, &mut ImageNode), Without<InventoryNext>>,
    mut qs: Query<(&mut Text, &mut Node, &InventoryStats)>,
    mut qin: Query<(&InventoryNext, &mut ImageNode), Without<Inventory>>,
    looking_gear: Res<LookingGear>,
) {
    for (ps, playergear) in q_gear.iter() {
        if gc.player_id == ps.id {
            for (inv, mut imgnode) in qi.iter_mut() {
                let gear = playergear.get_hand(&inv.hand);
                let idx = gear.get_sprite_idx() as usize;
                if imgnode.texture_atlas.as_ref().unwrap().index != idx {
                    imgnode.texture_atlas.as_mut().unwrap().index = idx;
                }
            }
            let left_hand_status = playergear.left_hand.get_status();
            let right_hand_status = playergear.right_hand.get_status();
            for (mut txt, mut node, istats) in qs.iter_mut() {
                let hand_status = match istats.hand {
                    Hand::Left => left_hand_status.clone(),
                    Hand::Right => right_hand_status.clone(),
                };
                let display = looking_gear.hand() == istats.hand;
                node.display = match display {
                    false => Display::None,
                    true => Display::Block,
                };
                if txt.0 != hand_status {
                    txt.0.clone_from(&hand_status);
                }
            }
            for (inv, mut imgnode) in qin.iter_mut() {
                // There are 2 possible "None" here, the outside Option::None for when the idx is
                // out of bounds and the inner Gear::None when a slot is empty.
                let next = if let Some(idx) = inv.idx {
                    playergear.get_next(idx).unwrap_or_default()
                } else {
                    playergear.get_next_non_empty().unwrap_or_default()
                };
                let idx = next.get_sprite_idx() as usize;
                if imgnode.texture_atlas.as_ref().unwrap().index != idx {
                    imgnode.texture_atlas.as_mut().unwrap().index = idx;
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(FixedUpdate, update_playerheld_gear_data)
        .add_systems(FixedUpdate, update_deployed_gear_data)
        .add_systems(FixedUpdate, update_deployed_gear_sprites)
        .add_systems(FixedUpdate, update_gear_ui)
        .add_systems(Update, keyboard_gear.run_if(in_state(GameState::None)))
        .add_systems(Update, sound_playback_system);
}
