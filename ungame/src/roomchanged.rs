use bevy::prelude::*;
use uncore::behaviour::Behaviour;
use uncore::behaviour::component::RoomState;
use uncore::components::board::position::Position;
use uncore::components::game::GCameraArena;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use uncore::states::GameState;
use unstd::systemparam::interactivestuff::InteractiveStuff;

/// Handles `RoomChangedEvent` events, updating interactive object states and room
/// data.
///
/// This system is responsible for:
///
/// * Updating the state of interactive objects based on the current room's state.
///
/// * Triggering the opening of the van UI when appropriate (e.g., when the player
///   enters the starting area).
///
/// * Updating the game's collision and lighting data after room-related changes.
fn roomchanged_event(
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut ev_room: EventReader<RoomChangedEvent>,
    mut interactive_stuff: InteractiveStuff,
    interactables: Query<(Entity, &Position, &Behaviour, &RoomState), Without<PlayerSprite>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform), Without<GCameraArena>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
) {
    let Some(ev) = ev_room.read().next() else {
        return;
    };
    for (entity, item_pos, behaviour, room_state) in interactables.iter() {
        let changed = interactive_stuff.execute_interaction(
            entity,
            item_pos,
            None,
            behaviour,
            Some(room_state),
            InteractionExecutionType::ReadRoomState,
        );
        if changed {
            // dbg!(&behaviour);
        }
    }
    ev_bdr.write(BoardDataToRebuild {
        lighting: true,
        collision: true,
    });
    if ev.open_van {
        interactive_stuff.game_next_state.set(GameState::Truck);
    }
    if ev.initialise {
        for (player, p_transform) in pc.iter() {
            if player.id != gc.player_id {
                continue;
            }
            for mut cam_trans in camera.iter_mut() {
                cam_trans.translation = p_transform.translation;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_event::<RoomChangedEvent>()
        .add_systems(Update, roomchanged_event);
}
