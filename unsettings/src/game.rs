use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq,
)]
pub struct GameplaySettings {
    pub movement_style: MovementStyle,
    pub camera_controls: CameraControls,
    pub character_controls: CharacterControls,
    pub dev_cheat_mode: DevCheatMode,
}

#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum GameplaySettingsValue {
    movement_style(MovementStyle),
    camera_controls(CameraControls),
    character_controls(CharacterControls),
    dev_cheat_mode(DevCheatMode),
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
pub enum MovementStyle {
    #[default]
    #[strum(to_string = "Isometric (Diagonal)")]
    Isometric,
    #[strum(to_string = "Orthogonal (Grid)")]
    ScreenSpaceOrthogonal,
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
pub enum CameraControls {
    #[default]
    #[strum(to_string = "Enabled")]
    On,
    #[strum(to_string = "Disabled")]
    Off,
}

impl CameraControls {
    pub fn on(&self) -> bool {
        match self {
            CameraControls::On => true,
            CameraControls::Off => false,
        }
    }
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
pub enum CharacterControls {
    #[default]
    WASD,
    Arrows,
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
pub enum DevCheatMode {
    #[default]
    #[strum(to_string = "Disabled")]
    Disabled,
    #[strum(to_string = "God Mode")]
    Enabled,
}

impl DevCheatMode {
    pub fn is_enabled(&self) -> bool {
        matches!(self, DevCheatMode::Enabled)
    }
}
