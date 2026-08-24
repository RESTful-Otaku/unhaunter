//! Menu definitions for the "Controls" settings category: device selection,
//! rebinding lists, stick sensitivity and related accessibility options.

use bevy::prelude::*;
use strum::IntoEnumIterator;
use unsettings::bindings::{
    ControlBindings, ControlSettingValue, PlayerAction, StickResponseCurve,
};

use crate::components::MenuEvent;

/// Which device's bindings are being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum BindDevice {
    Keyboard,
    Gamepad,
}

impl BindDevice {
    /// Current human-readable binding for an action on this device.
    fn binding_label(&self, bindings: &ControlBindings, action: PlayerAction) -> String {
        match self {
            Self::Keyboard => match bindings.key(action) {
                Some(k) => format!("{k:?}"),
                None => "-".to_string(),
            },
            Self::Gamepad => match bindings.button(action) {
                Some(b) => gamepad_button_label(b),
                None => "-".to_string(),
            },
        }
    }
}

/// Friendly label for standard gamepad buttons.
pub use uncore::input::gamepad_button_label;

/// Level-2 entries inside the Controls category.
#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSettingsMenu {
    #[strum(to_string = "Input Devices")]
    DeviceMode,
    #[strum(to_string = "Connected Gamepads")]
    ConnectedPads,
    #[strum(to_string = "Keyboard Bindings")]
    KeyboardBindings,
    #[strum(to_string = "Gamepad Bindings")]
    GamepadBindings,
    #[strum(to_string = "Stick Sensitivity")]
    StickSettings,
    #[strum(to_string = "Run Mode")]
    RunMode,
    #[strum(to_string = "Rumble Feedback")]
    Rumble,
    #[strum(to_string = "Rumble Strength")]
    RumbleIntensity,
    #[strum(to_string = "Reset Keyboard Defaults")]
    ResetKeyboard,
    #[strum(to_string = "Reset Gamepad Defaults")]
    ResetGamepad,
}

impl ControlSettingsMenu {
    /// The event triggered when this entry is activated.
    pub fn menu_event(&self) -> MenuEvent {
        match self {
            Self::DeviceMode => MenuEvent::EditControlSetting(ControlSettingsMenu::DeviceMode),
            Self::ConnectedPads => MenuEvent::BindingInfo,
            Self::KeyboardBindings => {
                MenuEvent::EditControlSetting(ControlSettingsMenu::KeyboardBindings)
            }
            Self::GamepadBindings => {
                MenuEvent::EditControlSetting(ControlSettingsMenu::GamepadBindings)
            }
            Self::StickSettings => {
                MenuEvent::EditControlSetting(ControlSettingsMenu::StickSettings)
            }
            Self::RunMode => MenuEvent::EditControlSetting(ControlSettingsMenu::RunMode),
            Self::Rumble => MenuEvent::EditControlSetting(ControlSettingsMenu::Rumble),
            Self::RumbleIntensity => {
                MenuEvent::EditControlSetting(ControlSettingsMenu::RumbleIntensity)
            }
            Self::ResetKeyboard => {
                MenuEvent::SaveControlSetting(ControlSettingValue::ResetKeyboard)
            }
            Self::ResetGamepad => MenuEvent::SaveControlSetting(ControlSettingValue::ResetGamepad),
        }
    }

    /// Current value summary shown next to the entry title.
    pub fn setting_value(&self, bindings: &ControlBindings) -> String {
        match self {
            Self::DeviceMode => bindings.device_mode.to_string(),
            Self::ConnectedPads => {
                // Placeholder; replaced by the caller that owns GamepadStatus.
                String::new()
            }
            Self::KeyboardBindings => format!("{} keys", bindings.keyboard.len()),
            Self::GamepadBindings => format!("{} buttons", bindings.gamepad.len()),
            Self::StickSettings => format!(
                "dz {:.2}/{:.2}",
                bindings.stick.move_deadzone, bindings.stick.aim_deadzone
            ),
            Self::RunMode => if bindings.run_is_toggle {
                "Toggle"
            } else {
                "Hold"
            }
            .to_string(),
            Self::Rumble => if bindings.rumble_enabled { "On" } else { "Off" }.to_string(),
            Self::RumbleIntensity => format!("{}%", (bindings.rumble_intensity * 100.0).round()),
            Self::ResetKeyboard | Self::ResetGamepad => String::new(),
        }
    }

    pub fn iter_events(bindings: &ControlBindings, pads_summary: &str) -> Vec<(String, MenuEvent)> {
        Self::iter()
            .map(|s| {
                let mut text = s.to_string();
                let value = match s {
                    Self::ConnectedPads => pads_summary.to_string(),
                    _ => s.setting_value(bindings),
                };
                if !value.is_empty() {
                    text = format!("{text}: {value}");
                }
                (text, s.menu_event())
            })
            .collect()
    }
}

/// Discrete steps offered for deadzone/sensitivity sliders.
const DEADZONE_STEPS: [f32; 8] = [0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40];
const SENSITIVITY_STEPS: [f32; 7] = [0.50, 0.75, 1.00, 1.25, 1.50, 1.75, 2.00];

fn stepped_options(
    current: f32,
    steps: &[f32],
    make: fn(f32) -> ControlSettingValue,
) -> Vec<(String, MenuEvent)> {
    steps
        .iter()
        .map(|v| {
            let label = if (current - v).abs() < f32::EPSILON {
                format!("[{v:.2}]")
            } else {
                format!("{v:.2}")
            };
            (label, MenuEvent::SaveControlSetting(make(*v)))
        })
        .collect()
}

/// Level-3 rows inside "Stick Sensitivity".
#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickSettingsMenu {
    #[strum(to_string = "Move Deadzone")]
    MoveDeadzone,
    #[strum(to_string = "Move Sensitivity")]
    MoveSensitivity,
    #[strum(to_string = "Aim Deadzone")]
    AimDeadzone,
    #[strum(to_string = "Aim Sensitivity")]
    AimSensitivity,
    #[strum(to_string = "Invert Aim Horizontal")]
    InvertAimX,
    #[strum(to_string = "Invert Aim Vertical")]
    InvertAimY,
    #[strum(to_string = "Response Curve")]
    ResponseCurve,
}

impl StickSettingsMenu {
    /// Builds every selectable row for the stick settings page: a header row
    /// (inert) followed by one option row per discrete value.
    pub fn build_rows(bindings: &ControlBindings) -> Vec<(String, MenuEvent)> {
        let stick = &bindings.stick;
        let mut rows: Vec<(String, MenuEvent)> = Vec::new();
        rows.extend(
            Self::MoveDeadzone
                .row_header(stick.move_deadzone)
                .into_iter()
                .chain(stepped_options(
                    stick.move_deadzone,
                    &DEADZONE_STEPS,
                    ControlSettingValue::MoveDeadzone,
                )),
        );
        rows.extend(
            Self::MoveSensitivity
                .row_header(stick.move_sensitivity)
                .into_iter()
                .chain(stepped_options(
                    stick.move_sensitivity,
                    &SENSITIVITY_STEPS,
                    ControlSettingValue::MoveSensitivity,
                )),
        );
        rows.extend(
            Self::AimDeadzone
                .row_header(stick.aim_deadzone)
                .into_iter()
                .chain(stepped_options(
                    stick.aim_deadzone,
                    &DEADZONE_STEPS,
                    ControlSettingValue::AimDeadzone,
                )),
        );
        rows.extend(
            Self::AimSensitivity
                .row_header(stick.aim_sensitivity)
                .into_iter()
                .chain(stepped_options(
                    stick.aim_sensitivity,
                    &SENSITIVITY_STEPS,
                    ControlSettingValue::AimSensitivity,
                )),
        );
        rows.push(bool_row("Invert Aim Horizontal", stick.invert_aim_x, |b| {
            ControlSettingValue::InvertAimX(b)
        }));
        rows.push(bool_row("Invert Aim Vertical", stick.invert_aim_y, |b| {
            ControlSettingValue::InvertAimY(b)
        }));
        for curve in StickResponseCurve::iter() {
            let label = if curve == stick.response_curve {
                format!("[{curve}]")
            } else {
                curve.to_string()
            };
            rows.push((
                format!("Response Curve: {label}"),
                MenuEvent::SaveControlSetting(ControlSettingValue::ResponseCurve(curve)),
            ));
        }
        rows
    }

    fn row_header(&self, value: f32) -> Vec<(String, MenuEvent)> {
        vec![(format!("{self}: {value:.2}"), MenuEvent::None)]
    }
}

fn bool_row(
    label: &str,
    value: bool,
    make: fn(bool) -> ControlSettingValue,
) -> (String, MenuEvent) {
    let new_value = !value;
    (
        format!("{label}: {}", if value { "On" } else { "Off" }),
        MenuEvent::SaveControlSetting(make(new_value)),
    )
}

/// Builds the rebindable-action list for one device.
pub fn rebind_list_rows(
    device: BindDevice,
    bindings: &ControlBindings,
) -> Vec<(String, MenuEvent)> {
    let mut rows = Vec::new();
    for action in PlayerAction::iter().filter(|a| a.is_gameplay()) {
        rows.push((
            format!(
                "{}: {}",
                action.label(),
                device.binding_label(bindings, action)
            ),
            MenuEvent::RebindRequest(device, action),
        ));
    }
    rows
}
