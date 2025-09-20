use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod display;
pub mod video_system;

#[derive(Component, Resource, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VideoSettings {
    pub resolution: display::Resolution,
    pub aspect_ratio: AspectRatio,
    pub ui_zoom: ZoomLevel,
    pub refresh_rate: RefreshRate,
    pub vsync: VSyncMode,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            resolution: display::Resolution::new(1920, 1080),
            aspect_ratio: AspectRatio::Auto,
            ui_zoom: ZoomLevel::Zoom100,
            refresh_rate: RefreshRate::Auto,
            vsync: VSyncMode::Auto,
        }
    }
}

#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum VideoSettingsValue {
    resolution(display::Resolution),
    aspect_ratio(AspectRatio),
    ui_zoom(ZoomLevel),
    refresh_rate(RefreshRate),
    vsync(VSyncMode),
}

// Re-export the AspectRatio from display module
pub use display::AspectRatio;

/// Represents UI zoom levels as percentages, similar to audio levels
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
)]
pub enum ZoomLevel {
    /// 50% zoom.
    #[strum(to_string = "50%")]
    Zoom050,
    /// 60% zoom.
    #[strum(to_string = "60%")]
    Zoom060,
    /// 70% zoom.
    #[strum(to_string = "70%")]
    Zoom070,
    /// 80% zoom.
    #[strum(to_string = "80%")]
    Zoom080,
    /// 90% zoom.
    #[strum(to_string = "90%")]
    Zoom090,
    /// 100% zoom (default).
    #[default]
    #[strum(to_string = "100%")]
    Zoom100,
    /// 110% zoom.
    #[strum(to_string = "110%")]
    Zoom110,
    /// 120% zoom.
    #[strum(to_string = "120%")]
    Zoom120,
    /// 130% zoom.
    #[strum(to_string = "130%")]
    Zoom130,
    /// 140% zoom.
    #[strum(to_string = "140%")]
    Zoom140,
    /// 150% zoom.
    #[strum(to_string = "150%")]
    Zoom150,
}

impl ZoomLevel {
    /// Converts the `ZoomLevel` to an `f32` zoom multiplier.
    pub fn as_f32(&self) -> f32 {
        match self {
            ZoomLevel::Zoom050 => 0.50,
            ZoomLevel::Zoom060 => 0.60,
            ZoomLevel::Zoom070 => 0.70,
            ZoomLevel::Zoom080 => 0.80,
            ZoomLevel::Zoom090 => 0.90,
            ZoomLevel::Zoom100 => 1.00,
            ZoomLevel::Zoom110 => 1.10,
            ZoomLevel::Zoom120 => 1.20,
            ZoomLevel::Zoom130 => 1.30,
            ZoomLevel::Zoom140 => 1.40,
            ZoomLevel::Zoom150 => 1.50,
        }
    }
}

/// Represents refresh rate settings
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
    Hash,
)]
pub enum RefreshRate {
    /// 30 Hz refresh rate
    #[strum(to_string = "30 Hz")]
    Hz30,
    /// 60 Hz refresh rate
    #[strum(to_string = "60 Hz")]
    Hz60,
    /// 75 Hz refresh rate
    #[strum(to_string = "75 Hz")]
    Hz75,
    /// 90 Hz refresh rate
    #[strum(to_string = "90 Hz")]
    Hz90,
    /// 120 Hz refresh rate
    #[strum(to_string = "120 Hz")]
    Hz120,
    /// 144 Hz refresh rate
    #[strum(to_string = "144 Hz")]
    Hz144,
    /// 165 Hz refresh rate
    #[strum(to_string = "165 Hz")]
    Hz165,
    /// 240 Hz refresh rate
    #[strum(to_string = "240 Hz")]
    Hz240,
    /// Auto (use display's preferred refresh rate)
    #[default]
    #[strum(to_string = "Auto")]
    Auto,
}

impl RefreshRate {
    /// Converts the `RefreshRate` to an `f32` Hz value.
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            RefreshRate::Hz30 => Some(30.0),
            RefreshRate::Hz60 => Some(60.0),
            RefreshRate::Hz75 => Some(75.0),
            RefreshRate::Hz90 => Some(90.0),
            RefreshRate::Hz120 => Some(120.0),
            RefreshRate::Hz144 => Some(144.0),
            RefreshRate::Hz165 => Some(165.0),
            RefreshRate::Hz240 => Some(240.0),
            RefreshRate::Auto => None,
        }
    }
}

/// Represents VSync settings
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Reflect,
    Component,
    strum::EnumIter,
    strum::Display,
    Hash,
)]
pub enum VSyncMode {
    /// VSync disabled
    #[strum(to_string = "Off")]
    Off,
    /// VSync enabled
    #[strum(to_string = "On")]
    On,
    /// Adaptive VSync (enables VSync when FPS is at or below refresh rate, disables when above)
    #[strum(to_string = "Adaptive")]
    Adaptive,
    /// Auto (use system default)
    #[default]
    #[strum(to_string = "Auto")]
    Auto,
}

impl VSyncMode {
    /// Converts the `VSyncMode` to a Bevy `PresentMode`.
    pub fn to_present_mode(&self) -> Option<bevy::window::PresentMode> {
        match self {
            VSyncMode::Off => Some(bevy::window::PresentMode::Immediate),
            VSyncMode::On => Some(bevy::window::PresentMode::AutoVsync),
            VSyncMode::Adaptive => Some(bevy::window::PresentMode::AutoNoVsync),
            VSyncMode::Auto => None,
        }
    }
}
