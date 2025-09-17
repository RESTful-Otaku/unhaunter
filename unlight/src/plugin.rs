use bevy::prelude::*;

use crate::{audio, maplight, metrics, player_light_level};

pub struct UnhaunterLightPlugin;

impl Plugin for UnhaunterLightPlugin {
    fn build(&self, app: &mut App) {
        audio::app_setup(app);
        maplight::app_setup(app);
        player_light_level::app_setup(app);
        metrics::register_all(app);
    }
}
