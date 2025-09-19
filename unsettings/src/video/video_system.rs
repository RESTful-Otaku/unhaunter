use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_persistent::Persistent;

/// Resource to track if video settings have changed and need to be applied
#[derive(Resource, Default)]
pub struct VideoSettingsChanged(pub bool);

/// System to apply video settings to the window and UI
pub fn apply_video_settings(
    mut windows: Query<&mut Window>,
    video_settings: Res<Persistent<crate::video::VideoSettings>>,
    mut settings_changed: ResMut<VideoSettingsChanged>,
    mut last_settings: Local<Option<crate::video::VideoSettings>>,
) {
    // Only apply settings if they have changed
    if !settings_changed.0 {
        return;
    }

    let mut window = match windows.single_mut() {
        Ok(window) => window,
        Err(_) => return,
    };

    // Check if settings actually changed
    let current_settings = (**video_settings).clone();
    if let Some(last) = last_settings.as_ref()
        && *last == current_settings {
            settings_changed.0 = false;
            return;
        }

    // Apply resolution changes
    let target_resolution = WindowResolution::new(
        video_settings.resolution.width as f32,
        video_settings.resolution.height as f32,
    );

    if window.resolution != target_resolution {
        window.resolution = target_resolution;
        info!("Applied resolution: {}x{}", video_settings.resolution.width, video_settings.resolution.height);
    }

    // Apply VSync changes
    if let Some(present_mode) = video_settings.vsync.to_present_mode() {
        window.present_mode = present_mode;
        info!("Applied VSync mode: {:?}", present_mode);
    }

    // Apply UI zoom scaling
    let zoom_factor = video_settings.ui_zoom.as_f32();
    info!("Applied UI zoom: {}%", (zoom_factor * 100.0) as u32);
    
    // Note: UI scaling should be handled by the UI system, not here
    // This is just for logging purposes

    // Store current settings and mark as applied
    *last_settings = Some(current_settings);
    settings_changed.0 = false;
}

/// System to handle aspect ratio changes
pub fn apply_aspect_ratio_settings(
    video_settings: Res<Persistent<crate::video::VideoSettings>>,
    settings_changed: ResMut<VideoSettingsChanged>,
) {
    // Only run if settings have changed
    if !settings_changed.0 {
        return;
    }

    if let Some(target_ratio) = video_settings.aspect_ratio.ratio() {
        let current_ratio = video_settings.resolution.aspect_ratio();
        
        if (current_ratio - target_ratio).abs() > 0.01 {
            // In a real implementation, this would adjust the viewport or letterboxing
            info!("Aspect ratio setting: {} (current: {:.2})", 
                  video_settings.aspect_ratio, current_ratio);
        }
    }
}

/// System to detect when video settings change and trigger reapplication
pub fn detect_video_settings_changes(
    video_settings: Res<Persistent<crate::video::VideoSettings>>,
    mut settings_changed: ResMut<VideoSettingsChanged>,
    mut last_settings: Local<Option<crate::video::VideoSettings>>,
) {
    let current_settings = (**video_settings).clone();
    
    if let Some(last) = last_settings.as_ref() {
        if *last != current_settings {
            info!("Video settings changed, marking for reapplication");
            settings_changed.0 = true;
        }
    } else {
        // First run, mark for initial application
        settings_changed.0 = true;
    }
    
    *last_settings = Some(current_settings);
}

/// System to apply UI scaling based on video settings
pub fn apply_ui_scaling(
    video_settings: Res<Persistent<crate::video::VideoSettings>>,
    settings_changed: ResMut<VideoSettingsChanged>,
    mut ui_scale: ResMut<UiScale>,
) {
    // Only apply if settings have changed
    if !settings_changed.0 {
        return;
    }

    let zoom_factor = video_settings.ui_zoom.as_f32();
    ui_scale.0 = zoom_factor;
    info!("Applied UI scale: {}%", (zoom_factor * 100.0) as u32);
}
