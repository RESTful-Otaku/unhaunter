use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq,
)]
pub struct VideoSettings {
    pub window_size: WindowSize,
    pub aspect_ratio: AspectRatio,
    pub ui_scale: Scale,
    pub font_scale: Scale,
}

#[derive(
    Reflect,
    Component,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::Display,
    strum::EnumIter,
)]
pub enum WindowSize {
    Small,
    #[default]
    Medium,
    Big,
}

#[derive(
    Reflect,
    Component,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::Display,
    strum::EnumIter,
)]
pub enum AspectRatio {
    Ar4_3,
    #[default]
    Ar16_10,
    Ar16_9,
}
#[derive(
    Reflect,
    Component,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::Display,
    strum::EnumIter,
)]
pub enum Scale {
    #[strum(to_string = "80%")]
    Scale080,
    #[strum(to_string = "90%")]
    Scale090,
    #[default]
    #[strum(to_string = "100%")]
    Scale100,
    #[strum(to_string = "110%")]
    Scale110,
    #[strum(to_string = "120%")]
    Scale120,
}

impl Scale {
    /// The multiplier this scale represents (e.g. `Scale090` -> `0.9`).
    pub fn as_f32(&self) -> f32 {
        match self {
            Scale::Scale080 => 0.80,
            Scale::Scale090 => 0.90,
            Scale::Scale100 => 1.00,
            Scale::Scale110 => 1.10,
            Scale::Scale120 => 1.20,
        }
    }
}

/// One menu-selected change to the video settings.
#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoSettingsValue {
    window_size(WindowSize),
    aspect_ratio(AspectRatio),
    ui_scale(Scale),
}

impl VideoSettingsValue {
    /// Applies this value to the given settings.
    pub fn apply(&self, settings: &mut VideoSettings) {
        match self {
            VideoSettingsValue::window_size(v) => settings.window_size = *v,
            VideoSettingsValue::aspect_ratio(v) => settings.aspect_ratio = *v,
            VideoSettingsValue::ui_scale(v) => settings.ui_scale = *v,
        }
    }
}

impl VideoSettings {
    /// Base window height in pixels for each size preset.
    pub fn resolution(&self) -> (f32, f32) {
        let height = match self.window_size {
            WindowSize::Small => 600.0,
            WindowSize::Medium => 800.0,
            WindowSize::Big => 1000.0,
        };
        let ratio = match self.aspect_ratio {
            AspectRatio::Ar4_3 => 4.0 / 3.0,
            AspectRatio::Ar16_10 => 1.6,
            AspectRatio::Ar16_9 => 16.0 / 9.0,
        };
        (height * ratio, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_multipliers_are_monotonic() {
        assert_eq!(Scale::Scale080.as_f32(), 0.80);
        assert_eq!(Scale::Scale100.as_f32(), 1.00);
        assert_eq!(Scale::Scale120.as_f32(), 1.20);
    }

    #[test]
    fn ui_scale_value_round_trips_through_settings() {
        let mut settings = VideoSettings::default();
        VideoSettingsValue::ui_scale(Scale::Scale120).apply(&mut settings);
        assert_eq!(settings.ui_scale, Scale::Scale120);
        VideoSettingsValue::ui_scale(Scale::Scale080).apply(&mut settings);
        assert_eq!(settings.ui_scale, Scale::Scale080);
    }
}
