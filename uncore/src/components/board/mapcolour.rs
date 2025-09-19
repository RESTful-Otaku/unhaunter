use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct MapColour {
    pub color: Color,
}
