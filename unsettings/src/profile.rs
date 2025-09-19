use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProfileSettings {
    pub display_name: String,
    pub color: Profilecolour,
}

#[derive(Reflect, Component, Serialize, Deserialize, Debug, Default, Clone, Copy, strum::Display, strum::EnumIter)]
pub enum Profilecolour {
    #[default]
    Grey,
    Red,
    Orange,
    Yellow,
    Lime,
    Green,
    Teal,
    Aqua,
    Blue,
    Violet,
    Purple,
}
