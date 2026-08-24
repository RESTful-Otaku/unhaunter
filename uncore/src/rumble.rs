//! Gamepad rumble feedback.
//!
//! Gameplay systems emit [`RumbleFeedback`] events; a single consumer system
//! translates them into Bevy [`GamepadRumbleRequest`]s for every connected
//! pad, scaled by the user's rumble intensity setting and gated by the
//! rumble-enabled toggle.

use std::time::Duration;

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::input::GamepadStatus;
use unsettings::bindings::ControlBindings;

/// A discrete haptic moment during gameplay.
#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub enum RumbleFeedback {
    /// Subtle confirmation (evidence recorded).
    Light,
    /// Interaction acknowledged (grab/drop gear).
    Medium,
    /// The ghost has started hunting.
    HuntStart,
    /// The hunt is over.
    HuntEnd,
    /// The player died.
    Death,
}

impl RumbleFeedback {
    /// `(duration, strong_motor, weak_motor)` before the intensity multiplier.
    fn profile(&self) -> (Duration, f32, f32) {
        match self {
            Self::Light => (Duration::from_millis(120), 0.15, 0.6),
            Self::Medium => (Duration::from_millis(180), 0.45, 0.5),
            Self::HuntStart => (Duration::from_millis(1200), 1.0, 0.5),
            Self::HuntEnd => (Duration::from_millis(500), 0.4, 0.4),
            Self::Death => (Duration::from_secs(2), 1.0, 0.8),
        }
    }
}

/// Translates gameplay [`RumbleFeedback`] events into rumble requests.
pub fn rumble_feedback_system(
    mut ev_feedback: EventReader<RumbleFeedback>,
    bindings: Res<Persistent<ControlBindings>>,
    gamepad_status: Res<GamepadStatus>,
    mut ev_rumble: EventWriter<GamepadRumbleRequest>,
) {
    if !bindings.rumble_enabled || bindings.rumble_intensity <= 0.0 {
        return;
    }
    let scale = bindings.rumble_intensity.clamp(0.0, 1.0);
    for feedback in ev_feedback.read() {
        let (duration, strong, weak) = feedback.profile();
        let intensity = GamepadRumbleIntensity {
            strong_motor: strong * scale,
            weak_motor: weak * scale,
        };
        for &gamepad in gamepad_status.pads.keys() {
            ev_rumble.write(GamepadRumbleRequest::Add {
                duration,
                intensity,
                gamepad,
            });
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_event::<RumbleFeedback>()
        .add_systems(PreUpdate, rumble_feedback_system);
}
