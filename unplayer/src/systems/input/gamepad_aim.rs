use bevy::prelude::*;
use uncore::{
    components::{
        board::{PERSPECTIVE_X, PERSPECTIVE_Y, direction::Direction},
        game::GCameraArena,
        player_sprite::PlayerSprite,
    },
    input::{ActionState, StickAimTracker},
};

/// Screen-space radius (pixels) mapped to a full stick deflection when aiming.
/// The arena camera uses a fixed 224px vertical viewport.
const STICK_AIM_SCREEN_RADIUS: f32 = 100.0;
/// Matches the mouse aim clamp distance in world units.
const AIM_MAX_DISTANCE: f32 = 8.0;

/// Converts a screen-space delta into a world-space direction on the isometric
/// plane. This is the same inverse projection used by `screen_to_world`, minus
/// the translation and fixed Z contributions (which cancel out for deltas).
fn screen_delta_to_world_direction(screen_delta: Vec2) -> Vec2 {
    let det = PERSPECTIVE_X[0] * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * PERSPECTIVE_X[1];
    if det.abs() < 1e-6 {
        return Vec2::ZERO;
    }
    let inv_det = 1.0 / det;
    Vec2::new(
        inv_det * (screen_delta.x * PERSPECTIVE_Y[1] - PERSPECTIVE_Y[0] * screen_delta.y),
        inv_det * (PERSPECTIVE_X[0] * screen_delta.y - screen_delta.x * PERSPECTIVE_X[1]),
    )
}

/// Aims the player's gear direction with the right analog stick.
///
/// Runs alongside [`super::mouse_aim_system`]; while the stick is deflected it
/// owns the facing direction (mouse aim yields because the cursor visibility
/// is suppressed by the stick-aim tracker).
pub fn gamepad_aim_system(
    actions: Res<ActionState>,
    mut q_player: Query<&mut Direction, With<PlayerSprite>>,
) {
    if !actions.stick_aiming {
        return;
    }
    // Scale the normalized stick vector to an on-screen offset from the
    // player, then project it onto the isometric world plane.
    let screen_offset = actions.aim_vector * STICK_AIM_SCREEN_RADIUS;
    let world_dir = screen_delta_to_world_direction(screen_offset);
    if world_dir == Vec2::ZERO {
        return;
    }
    let clamped = world_dir.clamp_length_max(AIM_MAX_DISTANCE) * 30.0;
    for mut dir in q_player.iter_mut() {
        *dir = Direction {
            dx: clamped.x,
            dy: clamped.y,
            dz: 0.0,
        };
    }
}

/// While the right stick aims, movement must not override the facing
/// direction; this helper is used by the movement system.
pub fn movement_should_control_facing(actions: &ActionState, tracker: &StickAimTracker) -> bool {
    !actions.stick_aiming && !tracker.is_active_within(0.25)
}
