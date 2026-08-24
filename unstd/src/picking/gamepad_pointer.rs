//! Gamepad-driven virtual pointer for in-game UI screens (truck, pause).
//!
//! Spawns a custom [`PointerId`] entity and feeds it synthetic
//! [`PointerInput`] events, so every UI that already works with the mouse
//! (buttons, tabs, sliders) becomes fully controllable with the analog stick.
//! A small ring is drawn at the virtual cursor position.

use bevy::prelude::*;
use bevy::render::camera::NormalizedRenderTarget;
use bevy::window::PrimaryWindow;
use bevy_persistent::Persistent;
use bevy_picking::{
    Pickable,
    pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput},
};
use uncore::colors;
use uncore::input::{ActionState, GamepadStatus, PlayerAction};
use uncore::states::{AppState, GameState};
use unsettings::bindings::{ControlBindings, InputDeviceMode};

/// Stable id for our software-controlled pointer.
pub fn gamepad_pointer_id() -> PointerId {
    PointerId::Custom(uuid::Uuid::from_u128(0x00C0_4711_57A7_D500))
}

/// Cursor travel speed at full stick deflection (pixels/second), before the
/// quadratic ease.
const CURSOR_SPEED: f32 = 1100.0;
/// Visual size of the virtual cursor ring.
const CURSOR_SIZE: f32 = 18.0;

#[derive(Resource, Debug, Default)]
struct GamepadCursorState {
    position: Vec2,
    active: bool,
}

/// Marker for the visual ring following the virtual cursor.
#[derive(Component)]
struct GamepadCursorVisual;

/// Adds the virtual gamepad pointer: a custom picking pointer driven by the
/// left analog stick, plus its on-screen ring.
pub struct GamepadPointerPlugin;

impl Plugin for GamepadPointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamepadCursorState>()
            .add_systems(Startup, spawn_gamepad_pointer)
            .add_systems(
                Update,
                (gamepad_pointer_input, update_cursor_visual).chain(),
            );
    }
}

fn spawn_gamepad_pointer(mut commands: Commands) {
    // The picking pipeline picks this up as a regular pointer; it only moves
    // when we send it synthetic events below.
    commands.spawn(gamepad_pointer_id());

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-2.0 * CURSOR_SIZE),
            top: Val::Px(-2.0 * CURSOR_SIZE),
            width: Val::Px(CURSOR_SIZE),
            height: Val::Px(CURSOR_SIZE),
            ..default()
        })
        .insert((
            BorderRadius::MAX,
            BorderColor(colors::TRUCKUI_ACCENT3_COLOR),
            BackgroundColor(Color::NONE),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            GlobalZIndex(10_000),
            GamepadCursorVisual,
        ));
}

fn window_location(window_entity: Entity, position: Vec2) -> Option<Location> {
    use bevy::window::WindowRef;
    let target = WindowRef::Entity(window_entity).normalize(Some(window_entity))?;
    Some(Location {
        target: NormalizedRenderTarget::Window(target),
        position,
    })
}

/// Whether the virtual cursor should be usable right now: a controller must be
/// connected and an interactive in-game screen must be open.
fn cursor_should_be_active(
    app_state: &AppState,
    game_state: &GameState,
    gamepad_status: &GamepadStatus,
    bindings: &ControlBindings,
) -> bool {
    if *app_state != AppState::InGame || !gamepad_status.is_any_connected() {
        return false;
    }
    if matches!(bindings.device_mode, InputDeviceMode::KeyboardAndMouse) {
        return false;
    }
    matches!(
        game_state,
        GameState::Truck | GameState::Pause | GameState::NpcHelp
    )
}

fn gamepad_pointer_input(
    actions: Res<ActionState>,
    bindings: Res<Persistent<ControlBindings>>,
    gamepad_status: Res<GamepadStatus>,
    windows: Query<(&Window, Entity), With<PrimaryWindow>>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut state: ResMut<GamepadCursorState>,
    time: Res<Time>,
    mut ev_pointer: EventWriter<PointerInput>,
) {
    let Ok((window, window_entity)) = windows.single() else {
        return;
    };
    let was_active = state.active;
    state.active = cursor_should_be_active(&app_state, &game_state, &gamepad_status, &bindings);

    let viewport = Vec2::new(window.width(), window.height());
    if state.active && !was_active {
        // Center the cursor when the cursor comes alive.
        state.position = Vec2::new(viewport.x * 0.5, viewport.y * 0.55);
    }
    if !state.active {
        if was_active {
            // Make sure a press is not left stuck when the screen closes.
            if let Some(location) = window_location(window_entity, state.position) {
                ev_pointer.write(PointerInput::new(
                    gamepad_pointer_id(),
                    location,
                    PointerAction::Release(PointerButton::Primary),
                ));
            }
        }
        return;
    }

    // Move with the left stick (movement systems are idle while these
    // overlays are open).
    let stick = actions.move_vector;
    if stick.length_squared() > 0.0 {
        // Quadratic response for precise small adjustments.
        let delta = stick * stick.length() * CURSOR_SPEED * time.delta_secs();
        state.position = (state.position + delta).clamp(Vec2::ZERO, viewport);
    }

    let Some(location) = window_location(window_entity, state.position) else {
        return;
    };

    if stick.length_squared() > 0.0 {
        ev_pointer.write(PointerInput::new(
            gamepad_pointer_id(),
            location.clone(),
            PointerAction::Move {
                delta: stick * 60.0,
            },
        ));
    }

    // A / Cross presses and releases whatever is under the ring. The press is
    // sent after movement so it lands on the freshly updated position.
    if actions.just_pressed(PlayerAction::Confirm) {
        ev_pointer.write(PointerInput::new(
            gamepad_pointer_id(),
            location.clone(),
            PointerAction::Press(PointerButton::Primary),
        ));
    }
    if actions.just_released(PlayerAction::Confirm) {
        ev_pointer.write(PointerInput::new(
            gamepad_pointer_id(),
            location,
            PointerAction::Release(PointerButton::Primary),
        ));
    }
}

fn update_cursor_visual(
    mut visual: Query<&mut Node, With<GamepadCursorVisual>>,
    state: Res<GamepadCursorState>,
) {
    for mut node in visual.iter_mut() {
        let (x, y) = if state.active {
            (
                state.position.x - CURSOR_SIZE * 0.5,
                state.position.y - CURSOR_SIZE * 0.5,
            )
        } else {
            (-2.0 * CURSOR_SIZE, -2.0 * CURSOR_SIZE)
        };
        node.left = Val::Px(x);
        node.top = Val::Px(y);
    }
}
