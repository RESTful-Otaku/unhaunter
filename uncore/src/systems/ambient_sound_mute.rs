use bevy::prelude::*;
use std::time::Duration;

use crate::{
    events::ambient_sound_mute::AmbientSoundMuteEvent,
    resources::ambient_mute::{ActiveMute, AmbientMuteController},
};

/// Processes ambient sound mute events and updates active mute timers.
/// Converts incoming mute events into active mutes and advances their timing state.
pub fn process_ambient_mute_events(
    mut mute_events: EventReader<AmbientSoundMuteEvent>,
    mut mute_controller: ResMut<AmbientMuteController>,
    time: Res<Time>,
) {
    // Add new mute requests to the controller
    for event in mute_events.read() {
        mute_controller.active_mutes.push(ActiveMute {
            config: event.clone(),
            elapsed: Duration::ZERO,
        });
    }

    // Update existing mutes and remove expired
    for mute in &mut mute_controller.active_mutes {
        mute.elapsed += time.delta();
    }
    mute_controller.active_mutes.retain(|m| !m.is_expired());
}
