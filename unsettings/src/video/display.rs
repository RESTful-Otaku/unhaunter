use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Re-export RefreshRate from the main video module
use crate::video::RefreshRate;

/// Resource that contains detected display information
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub available_resolutions: Vec<Resolution>,
    pub available_aspect_ratios: Vec<AspectRatio>,
    pub available_refresh_rates: Vec<RefreshRate>,
    pub primary_monitor_resolution: Resolution,
    pub primary_monitor_refresh_rate: RefreshRate,
}

/// Represents a specific screen resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

}

impl Default for Resolution {
    fn default() -> Self {
        Self { width: 1920, height: 1080 }
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Represents detected aspect ratios from the display
#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default,
    strum::Display, strum::EnumIter, Hash,
)]
pub enum AspectRatio {
    #[strum(to_string = "4:3")]
    Ar4_3,
    #[strum(to_string = "16:10")]
    Ar16_10,
    #[strum(to_string = "16:9")]
    Ar16_9,
    #[strum(to_string = "21:9")]
    Ar21_9,
    #[strum(to_string = "32:9")]
    Ar32_9,
    #[default]
    #[strum(to_string = "Auto")]
    Auto,
    Custom(u32, u32), // Store as numerator and denominator to avoid f32 issues
}

impl AspectRatio {
    pub fn from_ratio(ratio: f32) -> Self {
        const TOLERANCE: f32 = 0.01;
        
        if (ratio - 4.0/3.0).abs() < TOLERANCE {
            AspectRatio::Ar4_3
        } else if (ratio - 16.0/10.0).abs() < TOLERANCE {
            AspectRatio::Ar16_10
        } else if (ratio - 16.0/9.0).abs() < TOLERANCE {
            AspectRatio::Ar16_9
        } else if (ratio - 21.0/9.0).abs() < TOLERANCE {
            AspectRatio::Ar21_9
        } else if (ratio - 32.0/9.0).abs() < TOLERANCE {
            AspectRatio::Ar32_9
        } else {
            // Convert to a reasonable fraction representation
            let numerator = (ratio * 1000.0).round() as u32;
            let denominator = 1000;
            AspectRatio::Custom(numerator, denominator)
        }
    }

    pub fn ratio(&self) -> Option<f32> {
        match self {
            AspectRatio::Ar4_3 => Some(4.0 / 3.0),
            AspectRatio::Ar16_10 => Some(16.0 / 10.0),
            AspectRatio::Ar16_9 => Some(16.0 / 9.0),
            AspectRatio::Ar21_9 => Some(21.0 / 9.0),
            AspectRatio::Ar32_9 => Some(32.0 / 9.0),
            AspectRatio::Auto => None,
            AspectRatio::Custom(num, den) => Some(*num as f32 / *den as f32),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            AspectRatio::Ar4_3 => "4:3".to_string(),
            AspectRatio::Ar16_10 => "16:10".to_string(),
            AspectRatio::Ar16_9 => "16:9".to_string(),
            AspectRatio::Ar21_9 => "21:9".to_string(),
            AspectRatio::Ar32_9 => "32:9".to_string(),
            AspectRatio::Auto => "Auto".to_string(),
            AspectRatio::Custom(num, den) => {
                let ratio = *num as f32 / *den as f32;
                format!("{:.2}:1", ratio)
            }
        }
    }
}

// Display implementation is provided by strum::Display derive

/// System to detect available display resolutions and aspect ratios
pub fn detect_display_info(
    mut commands: Commands,
    windows: Query<&Window>,
) {
    let mut available_resolutions = Vec::new();
    let mut aspect_ratio_set = HashSet::new();
    let mut refresh_rate_set = HashSet::new();
    let mut primary_resolution = Resolution::new(1920, 1080); // fallback
    let mut primary_refresh_rate = RefreshRate::Hz60; // fallback

    // Get the primary window
    if let Ok(window) = windows.single() {
        let current_resolution = Resolution::new(
            window.resolution.width() as u32,
            window.resolution.height() as u32,
        );
        primary_resolution = current_resolution;
        available_resolutions.push(current_resolution);
        aspect_ratio_set.insert(AspectRatio::from_ratio(primary_resolution.aspect_ratio()));
        
        // Try to detect refresh rate from window
        // Note: Bevy doesn't directly expose refresh rate, so we'll use common values
        primary_refresh_rate = RefreshRate::Hz60; // Default assumption
    }

    // Add common resolutions that are typically supported
    let common_resolutions = vec![
        Resolution::new(640, 480),    // VGA
        Resolution::new(800, 600),    // SVGA
        Resolution::new(1024, 768),   // XGA
        Resolution::new(1280, 720),   // HD
        Resolution::new(1280, 800),   // WXGA
        Resolution::new(1280, 1024),  // SXGA
        Resolution::new(1366, 768),   // HD
        Resolution::new(1440, 900),   // WXGA+
        Resolution::new(1600, 900),   // HD+
        Resolution::new(1600, 1200),  // UXGA
        Resolution::new(1680, 1050),  // WSXGA+
        Resolution::new(1920, 1080),  // Full HD
        Resolution::new(1920, 1200),  // WUXGA
        Resolution::new(2048, 1152),  // QWXGA
        Resolution::new(2560, 1440),  // QHD
        Resolution::new(2560, 1600),  // WQXGA
        Resolution::new(3440, 1440),  // UWQHD
        Resolution::new(3840, 2160),  // 4K UHD
        Resolution::new(5120, 1440),  // Dual QHD
        Resolution::new(5120, 2880),  // 5K
        Resolution::new(7680, 4320),  // 8K UHD
    ];

    for resolution in common_resolutions {
        if !available_resolutions.contains(&resolution) {
            available_resolutions.push(resolution);
        }
        aspect_ratio_set.insert(AspectRatio::from_ratio(resolution.aspect_ratio()));
    }

    // Add common refresh rates
    let common_refresh_rates = vec![
        RefreshRate::Hz30,
        RefreshRate::Hz60,
        RefreshRate::Hz75,
        RefreshRate::Hz90,
        RefreshRate::Hz120,
        RefreshRate::Hz144,
        RefreshRate::Hz165,
        RefreshRate::Hz240,
        RefreshRate::Auto,
    ];

    for refresh_rate in common_refresh_rates {
        refresh_rate_set.insert(refresh_rate);
    }

    // Sort resolutions by total pixel count (width * height)
    available_resolutions.sort_by_key(|r| r.width * r.height);

    let available_aspect_ratios: Vec<AspectRatio> = aspect_ratio_set.into_iter().collect();
    let available_refresh_rates: Vec<RefreshRate> = refresh_rate_set.into_iter().collect();

    let display_info = DisplayInfo {
        available_resolutions,
        available_aspect_ratios,
        available_refresh_rates,
        primary_monitor_resolution: primary_resolution,
        primary_monitor_refresh_rate: primary_refresh_rate,
    };

    info!("Detected display info: {} resolutions, {} aspect ratios, {} refresh rates", 
          display_info.available_resolutions.len(), 
          display_info.available_aspect_ratios.len(),
          display_info.available_refresh_rates.len());

    commands.insert_resource(display_info);
}

/// System to update video settings based on detected display info
pub fn update_video_settings_from_display(
    display_info: Res<DisplayInfo>,
    mut video_settings: ResMut<bevy_persistent::Persistent<crate::video::VideoSettings>>,
) {
    // If the current resolution is not in the available list, set it to the primary monitor resolution
    if !display_info.available_resolutions.contains(&video_settings.resolution) {
        video_settings.resolution = display_info.primary_monitor_resolution;
    }
}
