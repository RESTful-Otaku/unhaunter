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

#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum VideoSettingsValue {
    window_size(WindowSize),
    aspect_ratio(AspectRatio),
    ui_scale(Scale),
    font_scale(Scale),
}

#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
    strum::Display, strum::EnumIter,
)]
pub enum WindowSize {
    Small,
    #[default]
    Medium,
    Big,
}

#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
    strum::Display, strum::EnumIter,
)]
pub enum AspectRatio {
    Ar4_3,
    #[default]
    Ar16_10,
    Ar16_9,
}
#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
    strum::Display, strum::EnumIter,
)]
pub enum Scale {
    Scale080,
    Scale090,
    #[default]
    Scale100,
    Scale110,
    Scale120,
}
