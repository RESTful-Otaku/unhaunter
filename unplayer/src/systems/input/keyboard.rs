use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::{
    components::{
        move_to::MoveToTarget,
        player_sprite::PlayerSprite,
        waypoint::{Waypoint, WaypointOwner, WaypointQueue},
    },
    input::{ActionState, PlayerAction},
    resources::player_input::PlayerInput,
};
use unsettings::game::{GameplaySettings, MovementStyle};

/// System that merges all movement sources into the [`PlayerInput`] resource.
///
/// Reads the analog/digital movement vector from the action-based
/// [`ActionState`] (which already merges keyboard and gamepad), applies the
/// configured [`MovementStyle`] transformation, and clears any active
/// click-to-move targets and waypoint queues when direct movement is used.
pub fn keyboard_input_system(
    actions: Res<ActionState>,
    mut commands: Commands,
    mut player_input: ResMut<PlayerInput>,
    players: Query<(Entity, &PlayerSprite)>,
    mut waypoint_queues: Query<&mut WaypointQueue>,
    q_existing_waypoints: Query<Entity, (With<Waypoint>, With<WaypointOwner>)>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for (entity, _player) in players.iter() {
        let mut movement = actions.move_vector;

        // Apply MovementStyle transformation (e.g. screen-space orthogonal).
        if matches!(
            game_settings.movement_style,
            MovementStyle::ScreenSpaceOrthogonal
        ) {
            const PERSPECTIVE_X: [f32; 2] = [1.0, 1.0];
            const PERSPECTIVE_Y: [f32; 2] = [-1.0, 1.0];
            let od = movement;
            movement.x = od.x * PERSPECTIVE_X[0] + od.y * PERSPECTIVE_Y[0];
            movement.y = od.x * PERSPECTIVE_X[1] + od.y * PERSPECTIVE_Y[1];
        }

        if movement != Vec2::ZERO {
            // Preserve analog magnitude; keyboard stays at unit length.
            // Clear any click-to-move target when using direct movement.
            commands.entity(entity).remove::<MoveToTarget>();

            // Clear waypoint queue and despawn waypoint entities
            if let Ok(mut waypoint_queue) = waypoint_queues.get_mut(entity) {
                // Despawn all waypoint entities belonging to this player
                for waypoint_entity in &waypoint_queue.0 {
                    if q_existing_waypoints.contains(*waypoint_entity) {
                        commands.entity(*waypoint_entity).despawn();
                    }
                }
                waypoint_queue.clear();
            }
        }

        player_input.movement = movement;
    }
}
