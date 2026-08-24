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
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
)]
pub enum Scale {
    Scale080,
    Scale090,
    #[default]
    Scale100,
    Scale110,
    Scale120,
}

/// One menu-selected change to the video settings.
#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoSettingsValue {
    window_size(WindowSize),
    aspect_ratio(AspectRatio),
}

impl VideoSettingsValue {
    /// Applies this value to the given settings.
    pub fn apply(&self, settings: &mut VideoSettings) {
        match self {
            VideoSettingsValue::window_size(v) => settings.window_size = *v,
            VideoSettingsValue::aspect_ratio(v) => settings.aspect_ratio = *v,
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
