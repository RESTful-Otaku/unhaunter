use bevy::prelude::*;
use bevy_persistent::Persistent;
use strum::IntoEnumIterator;
use unsettings::{
    audio::{AudioLevel, AudioSettings, AudioSettingsValue},
    game::{CameraControls, DevCheatMode, GameplaySettings, GameplaySettingsValue, MovementStyle},
    profile::{ProfileSettings, Profilecolour},
    video::{VideoSettings, VideoSettingsValue, display::Resolution, AspectRatio, ZoomLevel},
};

#[expect(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum ProfileSettingsValue {
    display_name(String),
    colour(Profilecolour),
}

use crate::components::MenuEvent;

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MenuSettingsLevel1 {
    Gameplay,
    Video,
    Audio,
    Profile,
}

impl MenuSettingsLevel1 {
    pub fn menu_event(&self) -> MenuEvent {
        use MenuSettingsLevel1 as m;
        match self {
            MenuSettingsLevel1::Gameplay => MenuEvent::SettingClassSelected(m::Gameplay),
            MenuSettingsLevel1::Audio => MenuEvent::SettingClassSelected(m::Audio),
            MenuSettingsLevel1::Video => MenuEvent::SettingClassSelected(m::Video),
            MenuSettingsLevel1::Profile => MenuEvent::SettingClassSelected(m::Profile),
        }
    }

    pub fn iter_events() -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| {
                let display_name = match s {
                    MenuSettingsLevel1::Gameplay => "Gameplay & Controls",
                    MenuSettingsLevel1::Audio => "Audio & Sound",
                    MenuSettingsLevel1::Video => "Graphics & Display",
                    MenuSettingsLevel1::Profile => "Player Profile",
                };
                (display_name.to_string(), s.menu_event())
            })
            .collect::<Vec<_>>()
    }
}

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSettingsMenu {
    #[strum(to_string = "Master Volume")]
    VolumeMaster,
    #[strum(to_string = "Background Music")]
    VolumeMusic,
    #[strum(to_string = "Sound Effects")]
    VolumeEffects,
    #[strum(to_string = "Ambient Sounds")]
    VolumeAmbient,
    #[strum(to_string = "Voice Chat")]
    VolumeVoiceChat,
    #[strum(to_string = "Audio Output")]
    SoundOutput,
    #[strum(to_string = "Spatial Audio")]
    AudioPositioning,
    #[strum(to_string = "Audio Latency")]
    FeedbackDelay,
    #[strum(to_string = "Audio Enhancement")]
    FeedbackEq,
}

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoSettingsMenu {
    #[strum(to_string = "Resolution")]
    WindowSize,
    #[strum(to_string = "Aspect Ratio")]
    AspectRatio,
    #[strum(to_string = "UI Scale")]
    UiScale,
    #[strum(to_string = "Font Size")]
    FontScale,
}

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSettingsMenu {
    #[strum(to_string = "Player Name")]
    DisplayName,
    #[strum(to_string = "Name Colour")]
    Colour,
}

impl VideoSettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        match self {
            VideoSettingsMenu::WindowSize => MenuEvent::EditVideoSetting(VideoSettingsMenu::WindowSize),
            VideoSettingsMenu::AspectRatio => MenuEvent::EditVideoSetting(VideoSettingsMenu::AspectRatio),
            VideoSettingsMenu::UiScale => MenuEvent::EditVideoSetting(VideoSettingsMenu::UiScale),
            VideoSettingsMenu::FontScale => MenuEvent::EditVideoSetting(VideoSettingsMenu::FontScale),
        }
    }

    pub fn iter_events(_video_settings: &VideoSettings) -> Vec<(String, MenuEvent)> {
        Self::iter()
            .map(|s| (s.to_string(), s.menu_event()))
            .collect::<Vec<_>>()
    }

    pub fn iter_events_item(&self, _video_settings: &VideoSettings) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        match self {
            VideoSettingsMenu::WindowSize => {
                // Common resolutions that users might want to choose from
                let common_resolutions = [
                    Resolution::new(1280, 720),   // HD
                    Resolution::new(1920, 1080),  // Full HD
                    Resolution::new(2560, 1440),  // QHD
                    Resolution::new(3840, 2160),  // 4K UHD
                ];
                common_resolutions.iter()
                    .map(|v| (v.to_string(), MenuEvent::SaveVideoSetting(VideoSettingsValue::resolution(*v))))
                    .collect()
            },
            VideoSettingsMenu::AspectRatio => AspectRatio::iter()
                .map(|v| (v.to_string(), MenuEvent::SaveVideoSetting(VideoSettingsValue::aspect_ratio(v))))
                .collect(),
            VideoSettingsMenu::UiScale => ZoomLevel::iter()
                .map(|v| (v.to_string(), MenuEvent::SaveVideoSetting(VideoSettingsValue::ui_zoom(v))))
                .collect(),
            VideoSettingsMenu::FontScale => ZoomLevel::iter()
                .map(|v| (v.to_string(), MenuEvent::SaveVideoSetting(VideoSettingsValue::ui_zoom(v))))
                .collect(),
        }
    }
}

impl ProfileSettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        match self {
            ProfileSettingsMenu::DisplayName => MenuEvent::EditProfileSetting(ProfileSettingsMenu::DisplayName),
            ProfileSettingsMenu::Colour => MenuEvent::EditProfileSetting(ProfileSettingsMenu::Colour),
        }
    }

    pub fn setting_value(&self, profile_settings: &Res<Persistent<ProfileSettings>>) -> String {
        match self {
            ProfileSettingsMenu::DisplayName => profile_settings.display_name.clone(),
            ProfileSettingsMenu::Colour => profile_settings.color.to_string(),
        }
    }

    pub fn iter_events_item(&self, profile_settings: &ProfileSettings) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        match self {
            ProfileSettingsMenu::DisplayName => {
                let mut options = vec![
                    ("Player".to_string(), MenuEvent::SaveProfileSetting(ProfileSettingsValue::display_name("Player".to_string()))),
                    ("Ghost Hunter".to_string(), MenuEvent::SaveProfileSetting(ProfileSettingsValue::display_name("Ghost Hunter".to_string()))),
                    ("Investigator".to_string(), MenuEvent::SaveProfileSetting(ProfileSettingsValue::display_name("Investigator".to_string()))),
                    ("Paranormal Expert".to_string(), MenuEvent::SaveProfileSetting(ProfileSettingsValue::display_name("Paranormal Expert".to_string()))),
                ];
                
                // Add the current custom name if it's not empty and not one of the presets
                let current_name = profile_settings.display_name.clone();
                if !current_name.is_empty() && !options.iter().any(|(name, _)| name == &current_name) {
                    // Add the custom name as a selectable option
                    options.push((format!("{} (Custom)", current_name), MenuEvent::SaveProfileSetting(ProfileSettingsValue::display_name(current_name.clone()))));
                    // Add a delete option for the custom name
                    options.push((format!("Delete '{}'", current_name), MenuEvent::DeleteCustomName(current_name)));
                }
                
                // Always add the custom input option at the end
                options.push(("Custom Name...".to_string(), MenuEvent::StartCustomNameInput));
                
                options
            }
            ProfileSettingsMenu::Colour => Profilecolour::iter()
                .map(|v| (v.to_string(), MenuEvent::SaveProfileSetting(ProfileSettingsValue::colour(v))))
                .collect(),
        }
    }

    pub fn iter_events(
        profile_settings: &Res<Persistent<ProfileSettings>>,
    ) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| {
                (
                    format!("{}: {}", s, s.setting_value(profile_settings)),
                    s.menu_event(),
                )
            })
            .collect::<Vec<_>>()
    }
}

impl AudioSettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        match self {
            // <-- add here the events for specific menus
            Self::VolumeMaster
            | Self::VolumeEffects
            | Self::VolumeMusic
            | Self::VolumeAmbient
            | Self::VolumeVoiceChat => MenuEvent::EditAudioSetting(*self),
            Self::SoundOutput
            | Self::AudioPositioning
            | Self::FeedbackDelay
            | Self::FeedbackEq => MenuEvent::EditAudioSetting(*self),
        }
    }

    pub fn setting_value(&self, audio_settings: &Res<Persistent<AudioSettings>>) -> String {
        match self {
            AudioSettingsMenu::VolumeMaster => audio_settings.volume_master.to_string(),
            AudioSettingsMenu::VolumeMusic => audio_settings.volume_music.to_string(),
            AudioSettingsMenu::VolumeEffects => audio_settings.volume_effects.to_string(),
            AudioSettingsMenu::VolumeAmbient => audio_settings.volume_ambient.to_string(),
            AudioSettingsMenu::VolumeVoiceChat => audio_settings.volume_voice_chat.to_string(),
            AudioSettingsMenu::SoundOutput => audio_settings.sound_output.to_string(),
            AudioSettingsMenu::AudioPositioning => audio_settings.audio_positioning.to_string(),
            AudioSettingsMenu::FeedbackDelay => audio_settings.feedback_delay.to_string(),
            AudioSettingsMenu::FeedbackEq => audio_settings.feedback_eq.to_string(),
        }
    }

    pub fn iter_events_item(
        &self,
        audio_settings: &Res<Persistent<AudioSettings>>,
    ) -> Vec<(String, MenuEvent)> {
        let to_string = |s: AudioLevel, v: &AudioLevel| -> String {
            if s == *v {
                format!("[{s}]")
            } else {
                s.to_string()
            }
        };
        match self {
            AudioSettingsMenu::VolumeMaster => AudioLevel::iter()
                .map(|s| {
                    (
                        to_string(s, &audio_settings.volume_master),
                        MenuEvent::SaveAudioSetting(AudioSettingsValue::volume_master(s)),
                    )
                })
                .collect::<Vec<_>>(),
            AudioSettingsMenu::VolumeEffects => AudioLevel::iter()
                .map(|s| {
                    (
                        to_string(s, &audio_settings.volume_effects),
                        MenuEvent::SaveAudioSetting(AudioSettingsValue::volume_effects(s)),
                    )
                })
                .collect::<Vec<_>>(),
            AudioSettingsMenu::VolumeMusic => AudioLevel::iter()
                .map(|s| {
                    (
                        to_string(s, &audio_settings.volume_music),
                        MenuEvent::SaveAudioSetting(AudioSettingsValue::volume_music(s)),
                    )
                })
                .collect::<Vec<_>>(),
            AudioSettingsMenu::VolumeAmbient => AudioLevel::iter()
                .map(|s| {
                    (
                        to_string(s, &audio_settings.volume_ambient),
                        MenuEvent::SaveAudioSetting(AudioSettingsValue::volume_ambient(s)),
                    )
                })
                .collect::<Vec<_>>(),
            AudioSettingsMenu::VolumeVoiceChat => AudioLevel::iter()
                .map(|s| {
                    (
                        to_string(s, &audio_settings.volume_voice_chat),
                        MenuEvent::SaveAudioSetting(AudioSettingsValue::volume_voice_chat(s)),
                    )
                })
                .collect::<Vec<_>>(),
            AudioSettingsMenu::SoundOutput => {
                use unsettings::audio::SoundOutput;
                let to_string = |s: SoundOutput, v: &SoundOutput| -> String {
                    if s == *v {
                        format!("[{s}]")
                    } else {
                        s.to_string()
                    }
                };
                SoundOutput::iter()
                    .map(|s| {
                        (
                            to_string(s, &audio_settings.sound_output),
                            MenuEvent::SaveAudioSetting(AudioSettingsValue::sound_output(s)),
                        )
                    })
                    .collect::<Vec<_>>()
            }
            AudioSettingsMenu::AudioPositioning => {
                use unsettings::audio::AudioPositioning;
                let to_string = |s: AudioPositioning, v: &AudioPositioning| -> String {
                    if s == *v {
                        format!("[{s}]")
                    } else {
                        s.to_string()
                    }
                };
                AudioPositioning::iter()
                    .map(|s| {
                        (
                            to_string(s, &audio_settings.audio_positioning),
                            MenuEvent::SaveAudioSetting(AudioSettingsValue::audio_positioning(s)),
                        )
                    })
                    .collect::<Vec<_>>()
            }
            AudioSettingsMenu::FeedbackDelay => {
                use unsettings::audio::FeedbackDelay;
                let to_string = |s: FeedbackDelay, v: &FeedbackDelay| -> String {
                    if s == *v {
                        format!("[{s}]")
                    } else {
                        s.to_string()
                    }
                };
                FeedbackDelay::iter()
                    .map(|s| {
                        (
                            to_string(s, &audio_settings.feedback_delay),
                            MenuEvent::SaveAudioSetting(AudioSettingsValue::feedback_delay(s)),
                        )
                    })
                    .collect::<Vec<_>>()
            }
            AudioSettingsMenu::FeedbackEq => {
                use unsettings::audio::FeedbackEQ;
                let to_string = |s: FeedbackEQ, v: &FeedbackEQ| -> String {
                    if s == *v {
                        format!("[{s}]")
                    } else {
                        s.to_string()
                    }
                };
                FeedbackEQ::iter()
                    .map(|s| {
                        (
                            to_string(s, &audio_settings.feedback_eq),
                            MenuEvent::SaveAudioSetting(AudioSettingsValue::feedback_eq(s)),
                        )
                    })
                    .collect::<Vec<_>>()
            }
        }
    }

    pub fn iter_events(
        audio_settings: &Res<Persistent<AudioSettings>>,
    ) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| {
                (
                    format!("{}: {}", s, s.setting_value(audio_settings)),
                    s.menu_event(),
                )
            })
            .collect::<Vec<_>>()
    }
}

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplaySettingsMenu {
    #[strum(to_string = "Movement Mode")]
    MovementStyle,
    #[strum(to_string = "Camera Movement")]
    CameraControls,
    #[strum(to_string = "Dev God Mode")]
    DevCheatMode,
}

impl GameplaySettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        match self {
            GameplaySettingsMenu::MovementStyle => MenuEvent::EditGameplaySetting(*self),
            GameplaySettingsMenu::CameraControls => MenuEvent::EditGameplaySetting(*self),
            GameplaySettingsMenu::DevCheatMode => MenuEvent::EditGameplaySetting(*self),
        }
    }

    pub fn setting_value(&self, game_settings: &Res<Persistent<GameplaySettings>>) -> String {
        match self {
            GameplaySettingsMenu::MovementStyle => game_settings.movement_style.to_string(),
            GameplaySettingsMenu::CameraControls => game_settings.camera_controls.to_string(),
            GameplaySettingsMenu::DevCheatMode => game_settings.dev_cheat_mode.to_string(),
        }
    }

    pub fn iter_events_item(
        &self,
        game_settings: &Res<Persistent<GameplaySettings>>,
    ) -> Vec<(String, MenuEvent)> {
        match self {
            GameplaySettingsMenu::MovementStyle => MovementStyle::iter()
                .map(|s| {
                    (
                        if s == game_settings.movement_style {
                            format!("[{s}]")
                        } else {
                            s.to_string()
                        },
                        MenuEvent::SaveGameplaySetting(GameplaySettingsValue::movement_style(s)),
                    )
                })
                .collect::<Vec<_>>(),
            GameplaySettingsMenu::CameraControls => CameraControls::iter()
                .map(|s| {
                    (
                        if s == game_settings.camera_controls {
                            format!("[{s}]")
                        } else {
                            s.to_string()
                        },
                        MenuEvent::SaveGameplaySetting(GameplaySettingsValue::camera_controls(s)),
                    )
                })
                .collect::<Vec<_>>(),
            GameplaySettingsMenu::DevCheatMode => DevCheatMode::iter()
                .map(|s| {
                    (
                        if s == game_settings.dev_cheat_mode {
                            format!("[{s}]")
                        } else {
                            s.to_string()
                        },
                        MenuEvent::SaveGameplaySetting(GameplaySettingsValue::dev_cheat_mode(s)),
                    )
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn iter_events(
        game_settings: &Res<Persistent<GameplaySettings>>,
    ) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| {
                (
                    format!("{}: {}", s, s.setting_value(game_settings)),
                    s.menu_event(),
                )
            })
            .collect::<Vec<_>>()
    }
}
