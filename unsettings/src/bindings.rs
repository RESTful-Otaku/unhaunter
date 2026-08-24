use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;
use enum_iterator::{Sequence, all};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Which input devices drive the game.
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::EnumIter,
    strum::Display,
)]
pub enum InputDeviceMode {
    /// All connected devices work simultaneously (recommended).
    #[default]
    Auto,
    /// Only keyboard and mouse are read.
    KeyboardAndMouse,
    /// Only gamepads are read. Falls back to keyboard if none is connected.
    Gamepad,
}

/// Response curve applied to analog stick deflection.
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::EnumIter,
    strum::Display,
)]
pub enum StickResponseCurve {
    /// Output grows linearly with deflection.
    #[default]
    Linear,
    /// Finer control near center, faster near rim. Recommended.
    Quadratic,
    /// Maximum finesse near center.
    Cubic,
}

/// All the player-facing actions that can be bound to inputs.
///
/// Each variant can be bound to one keyboard key and one gamepad button.
/// Movement/UI navigation additionally respond to the analog sticks.
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, strum::EnumIter,
)]
pub enum PlayerAction {
    // -- Gameplay --
    /// Move up.
    MoveUp,
    /// Move down.
    MoveDown,
    /// Move left.
    MoveLeft,
    /// Move right.
    MoveRight,
    /// Interact with objects / hide while held.
    Activate,
    /// Grab an object.
    Grab,
    /// Drop / deploy the held object.
    Drop,
    /// Use (trigger) the right-hand gear.
    TriggerRightHand,
    /// Toggle the left-hand gear (e.g. flashlight).
    TorchLeftHand,
    /// Cycle right-hand inventory item.
    CycleInventory,
    /// Swap left/right hand items.
    SwapHands,
    /// Record evidence in the quick journal.
    ChangeEvidence,
    /// Hold to run (or toggle when enabled).
    Run,
    /// Temporarily look at the left-hand gear (hold).
    LookLeftHandHold,
    /// Toggle looking at the left-hand gear.
    LookLeftHandToggle,

    // -- Camera --
    /// Pan the camera up.
    CameraUp,
    /// Pan the camera down.
    CameraDown,
    /// Pan the camera left.
    CameraLeft,
    /// Pan the camera right.
    CameraRight,
    /// Zoom the camera in.
    ZoomIn,
    /// Zoom the camera out.
    ZoomOut,

    // -- Menus / global --
    /// Navigate up in menus.
    MenuUp,
    /// Navigate down in menus.
    MenuDown,
    /// Navigate left in menus.
    MenuLeft,
    /// Navigate right in menus.
    MenuRight,
    /// Confirm / select in menus.
    Confirm,
    /// Go back / pause gameplay.
    Back,
}

impl PlayerAction {
    /// Stable index into fixed-size action tables (declaration order).
    pub fn ordinal(self) -> usize {
        self as usize
    }

    /// Human-readable label shown in the rebinding UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::MoveUp => "Move Up",
            Self::MoveDown => "Move Down",
            Self::MoveLeft => "Move Left",
            Self::MoveRight => "Move Right",
            Self::Activate => "Interact / Hide",
            Self::Grab => "Grab Item",
            Self::Drop => "Drop / Deploy",
            Self::TriggerRightHand => "Use Right Gear",
            Self::TorchLeftHand => "Toggle Left Gear",
            Self::CycleInventory => "Cycle Inventory",
            Self::SwapHands => "Swap Hands",
            Self::ChangeEvidence => "Record Evidence",
            Self::Run => "Run",
            Self::LookLeftHandHold => "Look At Left Gear (Hold)",
            Self::LookLeftHandToggle => "Look At Left Gear (Toggle)",
            Self::CameraUp => "Camera Up",
            Self::CameraDown => "Camera Down",
            Self::CameraLeft => "Camera Left",
            Self::CameraRight => "Camera Right",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::MenuUp => "Menu Up",
            Self::MenuDown => "Menu Down",
            Self::MenuLeft => "Menu Left",
            Self::MenuRight => "Menu Right",
            Self::Confirm => "Confirm",
            Self::Back => "Back / Pause",
        }
    }

    /// Whether this action belongs to the gameplay group.
    pub fn is_gameplay(&self) -> bool {
        matches!(
            self,
            Self::MoveUp
                | Self::MoveDown
                | Self::MoveLeft
                | Self::MoveRight
                | Self::Activate
                | Self::Grab
                | Self::Drop
                | Self::TriggerRightHand
                | Self::TorchLeftHand
                | Self::CycleInventory
                | Self::SwapHands
                | Self::ChangeEvidence
                | Self::Run
                | Self::LookLeftHandHold
                | Self::LookLeftHandToggle
        )
    }

    /// Whether this action has a default gamepad binding. Camera and zoom
    /// actions are keyboard-only: the analog sticks are reserved for movement
    /// and aiming.
    pub fn is_gamepad_bindable(&self) -> bool {
        !matches!(
            self,
            Self::CameraUp
                | Self::CameraDown
                | Self::CameraLeft
                | Self::CameraRight
                | Self::ZoomIn
                | Self::ZoomOut
        )
    }
}

/// Tuning knobs for analog sticks (accessibility & precision).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct StickSettings {
    /// Radial deadzone for the movement (left) stick, `0.0..=0.5`.
    pub move_deadzone: f32,
    /// Extra multiplier applied to movement stick output, `0.25..=2.0`.
    pub move_sensitivity: f32,
    /// Radial deadzone for the aiming (right) stick, `0.0..=0.5`.
    pub aim_deadzone: f32,
    /// Extra multiplier applied to aim stick output, `0.25..=2.0`.
    pub aim_sensitivity: f32,
    /// Invert the horizontal axis of the aim stick.
    pub invert_aim_x: bool,
    /// Invert the vertical axis of the aim stick.
    pub invert_aim_y: bool,
    /// Response curve applied to both sticks.
    pub response_curve: StickResponseCurve,
}

impl Default for StickSettings {
    fn default() -> Self {
        Self {
            move_deadzone: 0.15,
            move_sensitivity: 1.0,
            aim_deadzone: 0.20,
            aim_sensitivity: 1.0,
            invert_aim_x: false,
            invert_aim_y: false,
            response_curve: StickResponseCurve::Quadratic,
        }
    }
}

/// Persistent control bindings for every device.
///
/// Stored as `control_bindings.ron`. Missing map entries mean the action is
/// unbound; accessors [`ControlBindings::key`] / [`ControlBindings::button`]
/// return `Option` accordingly.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ControlBindings {
    /// Which devices feed the input pipeline.
    pub device_mode: InputDeviceMode,
    /// Keyboard key bound to each action.
    pub keyboard: HashMap<PlayerAction, KeyCode>,
    /// Gamepad button bound to each action.
    pub gamepad: HashMap<PlayerAction, GamepadButton>,
    /// Analog stick tuning.
    pub stick: StickSettings,
    /// Enable rumble feedback (when supported by the platform/device).
    pub rumble_enabled: bool,
    /// Rumble strength multiplier, from 0.0 (silent) to 1.0 (full).
    #[serde(default = "default_rumble_intensity")]
    pub rumble_intensity: f32,
    /// Treat the Run binding as a toggle instead of hold-to-run.
    pub run_is_toggle: bool,
}

fn default_rumble_intensity() -> f32 {
    0.7
}

impl Default for ControlBindings {
    fn default() -> Self {
        Self {
            device_mode: InputDeviceMode::Auto,
            keyboard: default_keyboard_bindings(),
            gamepad: default_gamepad_bindings(),
            stick: StickSettings::default(),
            rumble_enabled: true,
            rumble_intensity: default_rumble_intensity(),
            run_is_toggle: false,
        }
    }
}

fn default_keyboard_bindings() -> HashMap<PlayerAction, KeyCode> {
    use KeyCode::*;
    all::<PlayerAction>()
        .zip([
            // Gameplay
            KeyW,
            KeyS,
            KeyA,
            KeyD,
            KeyE,
            KeyF,
            KeyG,
            KeyR,
            Tab,
            KeyQ,
            KeyT,
            KeyC,
            ShiftLeft,
            ControlLeft,
            CapsLock,
            // Camera
            ArrowUp,
            ArrowDown,
            ArrowLeft,
            ArrowRight,
            NumpadAdd,
            NumpadSubtract,
            // Menus / global
            ArrowUp,
            ArrowDown,
            ArrowLeft,
            ArrowRight,
            Enter,
            Escape,
        ])
        .collect()
}

fn default_gamepad_bindings() -> HashMap<PlayerAction, GamepadButton> {
    all::<PlayerAction>()
        .zip([
            // Gameplay
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
            GamepadButton::South,         // A / Cross: interact & hide
            GamepadButton::West,          // X / Square: grab
            GamepadButton::East,          // B / Circle: drop/deploy
            GamepadButton::North,         // Y / Triangle: use right-hand gear
            GamepadButton::LeftTrigger2,  // LT: toggle left gear
            GamepadButton::LeftTrigger,   // LB: cycle inventory
            GamepadButton::RightTrigger,  // RB: swap hands
            GamepadButton::Select,        // Back/Share: record evidence
            GamepadButton::RightTrigger2, // RT: run
            GamepadButton::LeftThumb,     // L3: look at left gear (hold)
            GamepadButton::RightThumb,    // R3: look at left gear (toggle)
            // Camera: panning/zooming is done with the right stick by default;
            // these are intentionally left unbound unless the player overrides them.
            GamepadButton::Other(255),
            GamepadButton::Other(255),
            GamepadButton::Other(255),
            GamepadButton::Other(255),
            GamepadButton::Other(255),
            GamepadButton::Other(255),
            // Menus / global
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
            GamepadButton::South, // A: confirm
            GamepadButton::Start, // Start/Options: back / pause
        ])
        .filter(|(_, b)| !matches!(b, GamepadButton::Other(_)))
        .collect()
}

impl ControlBindings {
    /// The keyboard key bound to `action`, if any.
    pub fn key(&self, action: PlayerAction) -> Option<KeyCode> {
        self.keyboard
            .get(&action)
            .copied()
            .filter(|k| *k != KeyCode::NonConvert)
    }

    /// The gamepad button bound to `action`, if any.
    pub fn button(&self, action: PlayerAction) -> Option<GamepadButton> {
        self.gamepad.get(&action).copied()
    }

    /// Rebind `action` on the keyboard, clearing any other action using the same key.
    /// Returns the action that previously owned the key (if any).
    pub fn set_key(
        &mut self,
        action: PlayerAction,
        key: KeyCode,
    ) -> Option<(PlayerAction, KeyCode)> {
        let previous_owner = self
            .keyboard
            .iter()
            .find(|(a, k)| **a != action && **k == key)
            .map(|(a, _)| *a);
        if let Some(other) = previous_owner {
            self.keyboard.remove(&other);
        }
        self.keyboard.insert(action, key);
        previous_owner.map(|other| (other, key))
    }

    /// Rebind `action` on the gamepad, clearing any other action using the same button.
    /// Returns the action that previously owned the button (if any).
    pub fn set_button(
        &mut self,
        action: PlayerAction,
        button: GamepadButton,
    ) -> Option<(PlayerAction, GamepadButton)> {
        let previous_owner = self
            .gamepad
            .iter()
            .find(|(a, b)| **a != action && **b == button)
            .map(|(a, _)| *a);
        if let Some(other) = previous_owner {
            self.gamepad.remove(&other);
        }
        self.gamepad.insert(action, button);
        previous_owner.map(|other| (other, button))
    }

    /// Restore keyboard defaults.
    pub fn reset_keyboard(&mut self) {
        self.keyboard = default_keyboard_bindings();
    }

    /// Restore gamepad defaults.
    pub fn reset_gamepad(&mut self) {
        self.gamepad = default_gamepad_bindings();
    }
}

/// Applies a radial deadzone, response curve and sensitivity to a raw stick vector.
///
/// Returns a vector with the same direction whose magnitude is in `0.0..=1.0`.
pub fn process_stick(
    raw: Vec2,
    deadzone: f32,
    sensitivity: f32,
    curve: StickResponseCurve,
) -> Vec2 {
    let len = raw.length();
    if len <= deadzone || len <= f32::EPSILON {
        return Vec2::ZERO;
    }
    // Rescale from [deadzone..=1] to [0..=1], then apply the response curve.
    let t = ((len - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0);
    let curved_len = match curve {
        StickResponseCurve::Linear => t,
        StickResponseCurve::Quadratic => t * t,
        StickResponseCurve::Cubic => t * t * t,
    };
    raw / len * (curved_len * sensitivity).clamp(0.0, 1.0)
}

/// Convenience: the keyboard defaults are equivalent to the classic WASD preset.
pub fn wasd_keyboard_bindings() -> HashMap<PlayerAction, KeyCode> {
    default_keyboard_bindings()
}

/// A single adjustable control value, used by the settings menus to apply
/// changes to [`ControlBindings`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlSettingValue {
    DeviceMode(InputDeviceMode),
    MoveDeadzone(f32),
    AimDeadzone(f32),
    MoveSensitivity(f32),
    AimSensitivity(f32),
    InvertAimX(bool),
    InvertAimY(bool),
    ResponseCurve(StickResponseCurve),
    RunIsToggle(bool),
    RumbleEnabled(bool),
    RumbleIntensity(f32),
    ResetKeyboard,
    ResetGamepad,
}

impl ControlBindings {
    /// Applies one menu-selected value to these bindings.
    pub fn apply_setting(&mut self, value: ControlSettingValue) {
        match value {
            ControlSettingValue::DeviceMode(m) => self.device_mode = m,
            ControlSettingValue::MoveDeadzone(v) => self.stick.move_deadzone = v,
            ControlSettingValue::AimDeadzone(v) => self.stick.aim_deadzone = v,
            ControlSettingValue::MoveSensitivity(v) => self.stick.move_sensitivity = v,
            ControlSettingValue::AimSensitivity(v) => self.stick.aim_sensitivity = v,
            ControlSettingValue::InvertAimX(b) => self.stick.invert_aim_x = b,
            ControlSettingValue::InvertAimY(b) => self.stick.invert_aim_y = b,
            ControlSettingValue::ResponseCurve(c) => self.stick.response_curve = c,
            ControlSettingValue::RunIsToggle(b) => self.run_is_toggle = b,
            ControlSettingValue::RumbleEnabled(b) => self.rumble_enabled = b,
            ControlSettingValue::RumbleIntensity(v) => self.rumble_intensity = v.clamp(0.0, 1.0),
            ControlSettingValue::ResetKeyboard => self.keyboard = default_keyboard_bindings(),
            ControlSettingValue::ResetGamepad => self.gamepad = default_gamepad_bindings(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let bindings = ControlBindings::default();
        let ron = ron::ser::to_string(&bindings).unwrap();
        let parsed: ControlBindings = ron::de::from_str(&ron).unwrap();
        assert_eq!(bindings, parsed);
    }

    #[test]
    fn deserializing_old_file_fills_missing_actions_with_defaults() {
        // Simulate a file written by an older version missing some actions.
        let partial = "(device_mode: Auto, keyboard: {}, gamepad: {}, \
                       stick: (move_deadzone: 0.15, move_sensitivity: 1.0, aim_deadzone: 0.2, \
                       aim_sensitivity: 1.0, invert_aim_x: false, invert_aim_y: false, \
                       response_curve: Linear), rumble_enabled: true, run_is_toggle: false)";
        let parsed: ControlBindings = ron::de::from_str(partial).unwrap();
        assert!(parsed.keyboard.is_empty());
        // Unbound actions resolve to None instead of panicking.
        assert_eq!(parsed.key(PlayerAction::Activate), None);
    }

    #[test]
    fn rebind_clears_duplicate() {
        let mut bindings = ControlBindings::default();
        let old_key = bindings.key(PlayerAction::MoveUp).unwrap();
        // Binding MoveUp's key onto MoveDown must free MoveUp.
        let displaced = bindings.set_key(PlayerAction::MoveDown, old_key);
        assert_eq!(displaced.map(|(a, _)| a), Some(PlayerAction::MoveUp));
        assert_eq!(bindings.key(PlayerAction::MoveUp), None);
        assert_eq!(bindings.key(PlayerAction::MoveDown), Some(old_key));
    }

    #[test]
    fn stick_processing_respects_deadzone_and_clamps() {
        let dz = 0.2;
        assert_eq!(
            process_stick(Vec2::new(0.1, 0.0), dz, 1.0, StickResponseCurve::Linear),
            Vec2::ZERO
        );
        let out = process_stick(
            Vec2::new(5.0, 0.0), // overdrive input, like some pads report
            dz,
            1.0,
            StickResponseCurve::Linear,
        );
        assert!((out.x - 1.0).abs() < 1e-6);
        let half = process_stick(Vec2::new(0.6, 0.0), 0.0, 1.0, StickResponseCurve::Linear);
        assert!((half.x - 0.6).abs() < 1e-6);
    }

    #[test]
    fn all_actions_have_defaults() {
        let bindings = ControlBindings::default();
        for action in all::<PlayerAction>() {
            assert!(
                bindings.keyboard.contains_key(&action),
                "{action:?} missing key"
            );
            if !action.is_gamepad_bindable() {
                continue;
            }
            assert!(
                bindings.gamepad.contains_key(&action),
                "{action:?} missing gamepad button"
            );
        }
    }
}
