use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::window::{MonitorSelection, VideoModeSelection, WindowResolution};
use bevy_persistent::Persistent;
use std::time::Duration;
use uncampaign::plugin::UnhaunterCampaignPlugin;
use uncore::difficulty::CurrentDifficulty;
use uncore::plugin::UnhaunterCorePlugin;
use uncore::resources::cli_options::CliOptions;
use uncore::{platform::plt, resources::object_interaction::ObjectInteractionConfig};
use uncoremenu::plugin::UnhaunterCoreMenuPlugin;
use unfog::plugin::UnhaunterFogPlugin;
use ungame::plugin::UnhaunterGamePlugin;
use ungear::plugin::UnhaunterGearPlugin;
use ungearitems::plugin::UnhaunterGearItemsPlugin;
use unghost::plugin::UnhaunterGhostPlugin;
use unlight::plugin::UnhaunterLightPlugin;
use unmaphub::plugin::UnhaunterMapHubPlugin;
use unmapload::plugin::UnhaunterMapLoadPlugin;
use unmenu::plugin::UnhaunterMenuPlugin;
use unmenusettings::plugin::UnhaunterMenuSettingsPlugin;
use unnpc::plugin::UnhaunterNPCPlugin;
use unplayer::plugin::UnhaunterPlayerPlugin;
use unprofile::plugin::UnhaunterProfilePlugin;
use unsettings::plugin::UnhaunterSettingsPlugin;
use unsettings::video::{FullscreenMode, VSyncMode, VideoSettings};
use unstd::materials::{CustomMaterial1, UIPanelMaterial};
use unstd::picking::{CustomSpritePickingPlugin, TruckNavPlugin};
use unstd::plugins::board::UnhaunterBoardPlugin;
use unstd::plugins::manual::UnhaunterManualPlugin;
use unstd::plugins::root::UnhaunterRootPlugin;
use unsummary::summary::UnhaunterSummaryPlugin;
use untmxmap::plugin::UnhaunterTmxMapPlugin;
use untruck::plugin::UnhaunterTruckPlugin;
use unwalkie::plugin::UnhaunterWalkiePlugin;

pub fn app_run(cli_options: CliOptions) {
    let mut app = App::new();
    let log_level = cli_options.log_level();
    app.insert_resource(cli_options);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Unhaunter {}", plt::VERSION),
                    resolution: default_resolution(),
                    // Enabling VSync might make it easier in WASM? (It doesn't)
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: log_level,
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.04, 0.08, 0.14)))
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 15.0,
    )));

    // `--mute` silences all audio at startup for automated testing / QA runs.
    if cli_options.mute {
        app.insert_resource(bevy::audio::GlobalVolume::new(bevy::audio::Volume::SILENT));
    }

    app.init_resource::<CurrentDifficulty>()
        .init_resource::<ObjectInteractionConfig>();

    app.add_plugins(FrameTimeDiagnosticsPlugin::new(1024));
    // app.add_plugins(LogDiagnosticsPlugin::default());

    app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
        .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());

    // Add picking support for our custom sprites
    app.add_plugins((CustomSpritePickingPlugin, TruckNavPlugin));

    app.add_plugins((
        UnhaunterCorePlugin,
        UnhaunterRootPlugin,
        UnhaunterBoardPlugin,
        UnhaunterManualPlugin,
        UnhaunterSummaryPlugin,
        UnhaunterGearPlugin,
        UnhaunterGearItemsPlugin,
        UnhaunterMapHubPlugin,
        UnhaunterTruckPlugin,
        UnhaunterGamePlugin,
        UnhaunterPlayerPlugin,
        UnhaunterGhostPlugin,
        UnhaunterMenuPlugin,
        UnhaunterLightPlugin,
        UnhaunterNPCPlugin,
    ));
    app.add_plugins((
        UnhaunterTmxMapPlugin,
        UnhaunterSettingsPlugin,
        UnhaunterMenuSettingsPlugin,
        UnhaunterFogPlugin,
        UnhaunterWalkiePlugin,
        UnhaunterCoreMenuPlugin,
        UnhaunterMapLoadPlugin,
        UnhaunterCampaignPlugin,
        UnhaunterProfilePlugin,
    ));
    app.add_systems(Update, crate::report_timer::report_performance);
    app.add_systems(PostUpdate, (apply_video_settings, apply_ui_scale));
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_systems(Startup, set_window_icon);
    }
    app.run();
}

/// Applies persisted video settings (window size, aspect ratio, fullscreen and
/// VSync) to the primary window whenever they change.
fn apply_video_settings(
    video: Res<Persistent<VideoSettings>>,
    mut q_window: Query<&mut bevy::window::Window, With<bevy::window::PrimaryWindow>>,
) {
    if !video.is_changed() {
        return;
    }

    let (width, height) = video.resolution();
    let fullscreen = video.fullscreen;
    let vsync = video.vsync;
    for mut window in q_window.iter_mut() {
        // VSync preference.
        window.present_mode = match vsync {
            VSyncMode::Auto => bevy::window::PresentMode::AutoVsync,
            VSyncMode::On => bevy::window::PresentMode::Fifo,
            VSyncMode::Off => bevy::window::PresentMode::AutoNoVsync,
        };

        // Window mode. In fullscreen the OS controls the size, so only set an
        // explicit resolution when windowed.
        window.mode = match fullscreen {
            FullscreenMode::Windowed => bevy::window::WindowMode::Windowed,
            FullscreenMode::Borderless => {
                bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
            }
            FullscreenMode::Exclusive => bevy::window::WindowMode::Fullscreen(
                MonitorSelection::Primary,
                VideoModeSelection::Current,
            ),
        };
        if !fullscreen.is_fullscreen() {
            window.resolution.set(width, height);
        }
    }
    info!(
        "Applied video settings: {}x{} {:?} vsync={:?}",
        width as u32, height as u32, fullscreen, vsync
    );
}

/// Applies the persisted UI scale to Bevy's global [`UiScale`] resource. This
/// scales all UI layout (menus, HUD, truck computer) uniformly, giving players
/// a granular way to make the interface larger or smaller.
fn apply_ui_scale(video: Res<Persistent<VideoSettings>>, mut ui_scale: ResMut<bevy::ui::UiScale>) {
    if !video.is_changed() {
        return;
    }
    let scale = video.ui_scale.as_f32();
    if (ui_scale.0 - scale).abs() > f32::EPSILON {
        ui_scale.0 = scale;
        info!("Applied UI scale: {:.0}%", scale * 100.0);
    }
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width, height)
}

#[cfg(not(target_arch = "wasm32"))]
use bevy::winit::WinitWindows;

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
) {
    // This only works on native. WASM uses the HTML icon.
    {
        use winit::window::Icon;
        let Some(assets_path) = crate::utils::find_assets_directory() else {
            warn!("Assets directory not found.");
            return;
        };
        // here we use the `image` crate to load our icon data from a png file
        // this is not a very bevy-native solution, but it will do
        let Ok(img) = image::open(assets_path.join("favicon-512x512.png")) else {
            warn!("Failed to load icon image.");
            return;
        };

        let (icon_rgba, icon_width, icon_height) = {
            let image = img.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

        // do it for all windows
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    }
}
