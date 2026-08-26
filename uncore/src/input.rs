//! Action-based input layer.
//!
//! All devices (keyboard, gamepads) are merged into a single [`ActionState`]
//! resource that gameplay and UI systems read from. This makes adding a new
//! device or rebinding an existing one transparent to the rest of the game.
//!
//! The mapping between physical inputs and [`PlayerAction`]s lives in the
//! persistent [`ControlBindings`] resource (`control_bindings.ron`).
//!
//! Mouse handling stays bespoke on purpose: clicks drive picking/UI
//! interaction through Bevy's own systems, so only keyboard and gamepads are
//! routed through this layer.

use bevy::input::gamepad::{
    Gamepad, GamepadButton, GamepadButtonChangedEvent, GamepadConnection, GamepadConnectionEvent,
};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use enum_iterator::all;
use std::collections::HashMap;
pub use unsettings::bindings::{
    ControlBindings, InputDeviceMode, PlayerAction, StickResponseCurve, process_stick,
};

/// Capacity for per-action state arrays. Guarded by a unit test against
/// `PlayerAction`'s real cardinality.
const ACTION_CAPACITY: usize = 32;

/// Snapshot of all digital action states for this frame, plus analog vectors.
#[derive(Resource, Debug, Clone)]
pub struct ActionState {
    pressed: [bool; ACTION_CAPACITY],
    just_pressed: [bool; ACTION_CAPACITY],
    just_released: [bool; ACTION_CAPACITY],
    /// Analog movement intent (already deadzoned/curved), screen axes.
    pub move_vector: Vec2,
    /// Right-stick aim vector (deadzoned/curved/scaled), screen axes.
    pub aim_vector: Vec2,
    /// True while the aim stick is deflected past its deadzone this frame.
    pub stick_aiming: bool,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            pressed: [false; ACTION_CAPACITY],
            just_pressed: [false; ACTION_CAPACITY],
            just_released: [false; ACTION_CAPACITY],
            move_vector: Vec2::ZERO,
            aim_vector: Vec2::ZERO,
            stick_aiming: false,
        }
    }
}

impl ActionState {
    fn idx(action: PlayerAction) -> usize {
        let ord = action.ordinal();
        debug_assert!(ord < ACTION_CAPACITY, "PlayerAction exceeds capacity");
        ord
    }

    /// Is `action` currently held down?
    pub fn pressed(&self, action: PlayerAction) -> bool {
        self.pressed[Self::idx(action)]
    }

    /// Was `action` pressed down this exact frame?
    pub fn just_pressed(&self, action: PlayerAction) -> bool {
        self.just_pressed[Self::idx(action)]
    }

    /// Was `action` released this exact frame?
    pub fn just_released(&self, action: PlayerAction) -> bool {
        self.just_released[Self::idx(action)]
    }

    /// First action in `actions` that was pressed down this frame, if any.
    pub fn any_just_pressed(&self, actions: &[PlayerAction]) -> Option<PlayerAction> {
        actions.iter().copied().find(|a| self.just_pressed(*a))
    }
}

/// Tracks connected gamepads for detection UIs.
#[derive(Resource, Debug, Default, Clone)]
pub struct GamepadStatus {
    /// Human readable name of every currently connected pad, by entity.
    pub pads: HashMap<Entity, String>,
}

impl GamepadStatus {
    pub fn is_any_connected(&self) -> bool {
        !self.pads.is_empty()
    }

    pub fn summary(&self) -> String {
        if self.pads.is_empty() {
            "None detected".to_string()
        } else {
            let mut names: Vec<&str> = self.pads.values().map(String::as_str).collect();
            names.sort();
            names.join(", ")
        }
    }
}

/// Run-latch used when `run_is_toggle` is enabled.
#[derive(Resource, Debug, Default)]
struct ToggleLatches {
    run: bool,
}

/// Tracks how recently the aim stick was used, so other systems (e.g. cursor
/// visibility, movement-facing) can yield to gamepad aiming gracefully.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct StickAimTracker {
    /// Seconds elapsed since the right stick last deflected past deadzone.
    pub seconds_since_active: f32,
}

impl StickAimTracker {
    /// True when the stick aimed within this many seconds.
    pub fn is_active_within(&self, seconds: f32) -> bool {
        self.seconds_since_active < seconds
    }
}

/// Merges keyboard + gamepad input into [`ActionState`] every frame.
///
/// Runs in `PreUpdate` right after Bevy's raw input processing.
fn update_action_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    q_gamepads: Query<(Entity, &Gamepad)>,
    bindings: Res<Persistent<ControlBindings>>,
    mut state: ResMut<ActionState>,
    mut latches: ResMut<ToggleLatches>,
    mut stick_aim_tracker: ResMut<StickAimTracker>,
    time: Res<Time>,
    mut repeat: Local<StickMenuRepeat>,
) {
    let prev_pressed = state.pressed;
    *state = synthesize(&keyboard, &q_gamepads, &bindings, &mut latches);

    // Derive edge flags from the previous frame's state.
    let pressed_now = state.pressed;
    for (i, prev) in prev_pressed.iter().enumerate() {
        state.just_pressed[i] = pressed_now[i] && !*prev;
        state.just_released[i] = !pressed_now[i] && *prev;
    }

    // Accessibility: run toggle latch (needs post-edge processing).
    apply_run_toggle(&mut state, &bindings, &mut latches);

    // Track recent stick-aim usage for other systems.
    stick_aim_tracker.seconds_since_active = if state.stick_aiming {
        0.0
    } else {
        stick_aim_tracker.seconds_since_active + time.delta_secs()
    };

    // Push-a-direction-and-hold auto-repeat for menu navigation via sticks.
    repeat.update(&mut state, time.delta_secs());
}

/// Pure synthesis of digital states from the raw device inputs.
fn synthesize(
    keyboard: &ButtonInput<KeyCode>,
    q_gamepads: &Query<(Entity, &Gamepad)>,
    bindings: &Persistent<ControlBindings>,
    _latches: &mut ToggleLatches,
) -> ActionState {
    let mut out = ActionState::default();
    let mode = bindings.device_mode;
    let any_pad = q_gamepads.iter().next().is_some();
    let mut use_keyboard = matches!(
        mode,
        InputDeviceMode::Auto | InputDeviceMode::KeyboardAndMouse
    );
    let use_gamepad = matches!(mode, InputDeviceMode::Auto | InputDeviceMode::Gamepad) && any_pad;
    // Gamepad-only mode falls back to keyboard when no pad is connected, so
    // the player can never lock themselves out of the game.
    let keyboard_fallback = mode == InputDeviceMode::Gamepad && !any_pad;
    if keyboard_fallback {
        use_keyboard = true;
    }

    if use_keyboard {
        for action in all::<PlayerAction>() {
            if let Some(key) = bindings.key(action)
                && keyboard.pressed(key)
            {
                out.pressed[ActionState::idx(action)] = true;
            }
        }
    }

    if use_gamepad {
        for (_, pad) in q_gamepads.iter() {
            for action in all::<PlayerAction>() {
                if let Some(button) = bindings.button(action)
                    && pad.pressed(button)
                {
                    out.pressed[ActionState::idx(action)] = true;
                }
            }
        }
    }

    // -- Analog sticks --
    let stick = &bindings.stick;
    let mut move_vec = Vec2::ZERO;
    let mut aim_vec = Vec2::ZERO;
    if use_gamepad {
        for (_, pad) in q_gamepads.iter() {
            move_vec += process_stick(
                pad.left_stick(),
                stick.move_deadzone,
                stick.move_sensitivity,
                stick.response_curve,
            );
            // D-pad also contributes digital movement.
            move_vec += pad.dpad();

            let raw_aim = Vec2::new(
                pad.right_stick().x * if stick.invert_aim_x { -1.0 } else { 1.0 },
                pad.right_stick().y * if stick.invert_aim_y { -1.0 } else { 1.0 },
            );
            aim_vec += process_stick(
                raw_aim,
                stick.aim_deadzone,
                stick.aim_sensitivity,
                // Aim keeps a linear curve: sensitivity is applied downstream
                // and a quadratic response makes precise aiming feel mushy.
                StickResponseCurve::Linear,
            );
        }
    }

    if use_keyboard {
        // Directional keys contribute to movement as well (analog sources win
        // simply by adding; both are clamped below).
        let mut key_dir = Vec2::ZERO;
        if out.pressed[ActionState::idx(PlayerAction::MoveUp)] {
            key_dir.y += 1.0;
        }
        if out.pressed[ActionState::idx(PlayerAction::MoveDown)] {
            key_dir.y -= 1.0;
        }
        if out.pressed[ActionState::idx(PlayerAction::MoveLeft)] {
            key_dir.x -= 1.0;
        }
        if out.pressed[ActionState::idx(PlayerAction::MoveRight)] {
            key_dir.x += 1.0;
        }
        move_vec += key_dir;
    }

    out.move_vector = move_vec.clamp_length_max(1.0);
    out.stick_aiming = use_gamepad && aim_vec.length_squared() > 1e-6;
    out.aim_vector = aim_vec.clamp_length_max(1.0);

    out
}

/// Converts hold-to-run into toggle-to-run when configured.
fn apply_run_toggle(
    state: &mut ActionState,
    bindings: &Persistent<ControlBindings>,
    latches: &mut ToggleLatches,
) {
    if !bindings.run_is_toggle {
        latches.run = false;
        return;
    }
    let run_idx = ActionState::idx(PlayerAction::Run);
    if state.just_pressed[run_idx] {
        latches.run = !latches.run;
    }
    // While latched, report as held; suppress the release edge so consumers
    // see a clean continuous hold instead of flickering.
    state.pressed[run_idx] = latches.run;
    state.just_released[run_idx] = false;
}

/// Implements push-a-direction-and-hold auto-repeat for menu navigation using
/// the left stick / d-pad, mirroring how text cursors repeat when held.
#[derive(Debug, Default)]
struct StickMenuRepeat {
    direction: Vec2,
    cooldown: f32,
}

impl StickMenuRepeat {
    const FIRST_DELAY: f32 = 0.35;
    const REPEAT_DELAY: f32 = 0.15;
    const THRESHOLD: f32 = 0.5;

    fn update(&mut self, state: &mut ActionState, dt: f32) {
        let dir = Self::dominant_axis(state.move_vector);
        let idx_for = |d: Vec2| -> Option<usize> {
            if d.y > 0.0 {
                Some(ActionState::idx(PlayerAction::MenuUp))
            } else if d.y < 0.0 {
                Some(ActionState::idx(PlayerAction::MenuDown))
            } else if d.x < 0.0 {
                Some(ActionState::idx(PlayerAction::MenuLeft))
            } else if d.x > 0.0 {
                Some(ActionState::idx(PlayerAction::MenuRight))
            } else {
                None
            }
        };

        match idx_for(dir) {
            None => {
                self.direction = Vec2::ZERO;
                self.cooldown = 0.0;
            }
            Some(idx) => {
                if dir != self.direction {
                    // Fresh engagement fires immediately.
                    self.direction = dir;
                    self.cooldown = Self::FIRST_DELAY;
                    state.just_pressed[idx] = true;
                    state.pressed[idx] = true;
                } else {
                    self.cooldown -= dt;
                    state.pressed[idx] = true;
                    if self.cooldown <= 0.0 {
                        state.just_pressed[idx] = true;
                        self.cooldown = Self::REPEAT_DELAY;
                    }
                }
            }
        }
    }

    fn dominant_axis(v: Vec2) -> Vec2 {
        if v.length() < Self::THRESHOLD {
            return Vec2::ZERO;
        }
        if v.x.abs() >= v.y.abs() {
            Vec2::new(v.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, v.y.signum())
        }
    }
}

/// Keeps [`GamepadStatus`] up to date with hotplugged pads.
fn track_gamepad_connections(
    mut events: EventReader<GamepadConnectionEvent>,
    q_gamepads: Query<Entity, With<Gamepad>>,
    mut status: ResMut<GamepadStatus>,
) {
    let mut changed = false;
    for ev in events.read() {
        changed = true;
        match &ev.connection {
            GamepadConnection::Connected { name, .. } => {
                info!("Gamepad connected: {name}");
                status.pads.insert(ev.gamepad, name.clone());
            }
            GamepadConnection::Disconnected => {
                info!("Gamepad disconnected");
                status.pads.remove(&ev.gamepad);
            }
        }
    }
    if changed {
        // Drop entries whose entity despawned without an explicit event.
        status.pads.retain(|entity, _| q_gamepads.contains(*entity));
    }
}

/// Drains analog button events; the thresholded digital state we consume lives
/// directly on the `Gamepad` component, so these events are not needed yet.
fn drain_button_events(mut events: EventReader<GamepadButtonChangedEvent>) {
    events.clear();
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<ActionState>()
        .init_resource::<GamepadStatus>()
        .init_resource::<ToggleLatches>()
        .init_resource::<StickAimTracker>()
        .add_systems(
            PreUpdate,
            (
                update_action_state.after(bevy::input::InputSystem),
                track_gamepad_connections,
                drain_button_events,
            ),
        );
    crate::rumble::app_setup(app);
}

/// Friendly label for a standard gamepad button, e.g. "A / Cross".
pub fn gamepad_button_label(button: GamepadButton) -> String {
    match button {
        GamepadButton::South => "A / Cross".into(),
        GamepadButton::East => "B / Circle".into(),
        GamepadButton::North => "Y / Triangle".into(),
        GamepadButton::West => "X / Square".into(),
        GamepadButton::LeftTrigger => "LB / L1".into(),
        GamepadButton::LeftTrigger2 => "LT / L2".into(),
        GamepadButton::RightTrigger => "RB / R1".into(),
        GamepadButton::RightTrigger2 => "RT / R2".into(),
        GamepadButton::Select => "Back / Share".into(),
        GamepadButton::Start => "Start / Options".into(),
        GamepadButton::Mode => "Home".into(),
        GamepadButton::LeftThumb => "L3".into(),
        GamepadButton::RightThumb => "R3".into(),
        GamepadButton::DPadUp => "D-Pad Up".into(),
        GamepadButton::DPadDown => "D-Pad Down".into(),
        GamepadButton::DPadLeft => "D-Pad Left".into(),
        GamepadButton::DPadRight => "D-Pad Right".into(),
        other => format!("{other:?}"),
    }
}

/// Compact label for a gamepad button, suited for inline prompts: "[A]".
fn gamepad_button_short(button: GamepadButton) -> String {
    match button {
        GamepadButton::South => "A".into(),
        GamepadButton::East => "B".into(),
        GamepadButton::North => "Y".into(),
        GamepadButton::West => "X".into(),
        GamepadButton::LeftTrigger => "LB".into(),
        GamepadButton::LeftTrigger2 => "LT".into(),
        GamepadButton::RightTrigger => "RB".into(),
        GamepadButton::RightTrigger2 => "RT".into(),
        GamepadButton::Select => "Back".into(),
        GamepadButton::Start => "Start".into(),
        GamepadButton::Mode => "Home".into(),
        GamepadButton::LeftThumb => "L3".into(),
        GamepadButton::RightThumb => "R3".into(),
        GamepadButton::DPadUp => "D-Up".into(),
        GamepadButton::DPadDown => "D-Down".into(),
        GamepadButton::DPadLeft => "D-Left".into(),
        GamepadButton::DPadRight => "D-Right".into(),
        other => format!("{other:?}"),
    }
}

/// Compact human label for a keyboard key, e.g. "E", "ESC", "Shift".
pub fn key_label(key: KeyCode) -> String {
    match key {
        KeyCode::Escape => "ESC".into(),
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift".into(),
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl".into(),
        KeyCode::AltLeft | KeyCode::AltRight => "Alt".into(),
        _ => {
            let name = format!("{key:?}");
            name.strip_prefix("Key").unwrap_or(&name).to_string()
        }
    }
}

/// Whether prompts should show gamepad bindings for the current mode.
pub fn prefers_gamepad(bindings: &ControlBindings, status: Option<&GamepadStatus>) -> bool {
    match bindings.device_mode {
        InputDeviceMode::Gamepad => true,
        InputDeviceMode::KeyboardAndMouse => false,
        InputDeviceMode::Auto => status.is_some_and(|s| s.is_any_connected()),
    }
}

/// Bracketed prompt for `action` reflecting the active control scheme, e.g.
/// "[A]" when playing on a controller or "[E]" on keyboard. Falls back to the
/// other device's binding when the preferred one is unbound.
pub fn action_prompt(
    bindings: &ControlBindings,
    action: PlayerAction,
    status: Option<&GamepadStatus>,
) -> String {
    if prefers_gamepad(bindings, status)
        && let Some(b) = bindings.button(action)
    {
        return format!("[{}]", gamepad_button_short(b));
    }
    if let Some(k) = bindings.key(action) {
        return format!("[{}]", key_label(k));
    }
    if let Some(b) = bindings.button(action) {
        return format!("[{}]", gamepad_button_short(b));
    }
    format!("[{:?}]", action)
}

/// Navigation hint line for menus, derived from the live bindings.
pub fn menu_nav_help(bindings: &ControlBindings, status: &GamepadStatus) -> String {
    let up = action_prompt(bindings, PlayerAction::MoveUp, Some(status));
    let down = action_prompt(bindings, PlayerAction::MoveDown, Some(status));
    let confirm = action_prompt(bindings, PlayerAction::Confirm, Some(status));
    let back = action_prompt(bindings, PlayerAction::Back, Some(status));
    format!("{up}/{down}: Change    |    {confirm}: Select    |    {back}: Go Back")
}

/// Contextual control hint line for the truck computer screen, derived from
/// the live bindings and the active device.
///
/// While inside the truck the "cycle inventory" / "swap hands" bindings act as
/// previous/next tab, mirroring how console games repurpose shoulder buttons.
pub fn truck_nav_help(bindings: &ControlBindings, status: &GamepadStatus) -> String {
    let prev_tab = action_prompt(bindings, PlayerAction::CycleInventory, Some(status));
    let next_tab = action_prompt(bindings, PlayerAction::SwapHands, Some(status));
    let left = action_prompt(bindings, PlayerAction::MenuLeft, Some(status));
    let right = action_prompt(bindings, PlayerAction::MenuRight, Some(status));
    let select = action_prompt(bindings, PlayerAction::Confirm, Some(status));
    let leave = action_prompt(bindings, PlayerAction::Back, Some(status));
    if prefers_gamepad(bindings, Some(status)) {
        format!(
            "{prev_tab}/{next_tab} Tab    |    D-Pad / Stick: Move    |    \
             {select}: Select (hold)    |    {leave}: Leave Truck"
        )
    } else {
        format!(
            "{left}/{right} or {prev_tab}/{next_tab} Switch Tab    |    Click: Select    |    \
             {leave}: Leave Truck"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_capacity_is_sufficient() {
        assert!(
            enum_iterator::cardinality::<PlayerAction>() <= ACTION_CAPACITY,
            "PlayerAction grew past ACTION_CAPACITY; bump the constant"
        );
    }

    #[test]
    fn dominant_axis_picks_strongest_direction() {
        assert_eq!(StickMenuRepeat::dominant_axis(Vec2::new(0.9, 0.2)), Vec2::X);
        assert_eq!(
            StickMenuRepeat::dominant_axis(Vec2::new(0.2, -0.9)),
            -Vec2::Y
        );
        assert_eq!(
            StickMenuRepeat::dominant_axis(Vec2::new(0.2, 0.2)),
            Vec2::ZERO
        );
    }
}
