//! Custom picking backend for map sprites
//!
//! This module provides a custom picking backend for map sprites that use custom
//! materials instead of the standard `Sprite` component. It enables mouse interaction
//! with doors, switches, and other interactive map elements.

mod gamepad_pointer;
mod sprite_picking_backend;
mod truck_nav;

pub use gamepad_pointer::GamepadPointerPlugin;
pub use sprite_picking_backend::{
    CustomSpritePickingCamera, CustomSpritePickingMode, CustomSpritePickingPlugin,
    CustomSpritePickingSettings,
};
