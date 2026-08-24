use bevy::prelude::*;
use bevy_persistent::Persistent;
use strum::IntoEnumIterator;
use unsettings::{
    audio::{AudioLevel, AudioSettings, AudioSettingsValue},
    game::{CameraControls, GameplaySettings, GameplaySettingsValue, MovementStyle},
    video::{AspectRatio, VideoSettings, VideoSettingsValue, WindowSize},
};

use crate::components::MenuEvent;

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MenuSettingsLevel1 {
    Gameplay,
    Video,
    Audio,
    Profile,
    #[strum(to_string = "Controls")]
    Controls,
}

impl MenuSettingsLevel1 {
    pub fn menu_event(&self) -> MenuEvent {
        use MenuSettingsLevel1 as m;
        match self {
            MenuSettingsLevel1::Gameplay => MenuEvent::SettingClassSelected(m::Gameplay),
            MenuSettingsLevel1::Video => MenuEvent::SettingClassSelected(m::Video),
            MenuSettingsLevel1::Audio => MenuEvent::SettingClassSelected(m::Audio),
            MenuSettingsLevel1::Controls => MenuEvent::SettingClassSelected(m::Controls),
            // Profile stays hidden until there is something to apply it to.
            MenuSettingsLevel1::Profile => MenuEvent::None,
        }
    }

    pub fn iter_events() -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| (s.to_string(), s.menu_event()))
            .collect::<Vec<_>>()
    }
}

#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSettingsMenu {
    #[strum(to_string = "Master Volume")]
    VolumeMaster,
    #[strum(to_string = "Music Volume")]
    VolumeMusic,
    #[strum(to_string = "Effects Volume")]
    VolumeEffects,
    #[strum(to_string = "Ambient Volume")]
    VolumeAmbient,
    #[strum(to_string = "VoiceChat Volume")]
    VolumeVoiceChat,
    #[strum(to_string = "Sound Output")]
    SoundOutput,
    #[strum(to_string = "Audio Positioning")]
    AudioPositioning,
    #[strum(to_string = "Feedback Delay")]
    FeedbackDelay,
    #[strum(to_string = "Feedback EQ")]
    FeedbackEq,
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
            Self::SoundOutput => MenuEvent::EditAudioSetting(*self),
            _ => MenuEvent::None,
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
            _ => vec![],
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
    #[strum(to_string = "Movement Style")]
    MovementStyle,
    #[strum(to_string = "Camera Controls")]
    CameraControls,
}

impl GameplaySettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        match self {
            GameplaySettingsMenu::MovementStyle => MenuEvent::EditGameplaySetting(*self),
            GameplaySettingsMenu::CameraControls => MenuEvent::EditGameplaySetting(*self),
        }
    }

    pub fn setting_value(&self, game_settings: &Res<Persistent<GameplaySettings>>) -> String {
        match self {
            GameplaySettingsMenu::MovementStyle => game_settings.movement_style.to_string(),
            GameplaySettingsMenu::CameraControls => game_settings.camera_controls.to_string(),
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

/// Level-2 entries inside the Video category.
#[derive(strum::Display, strum::EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoSettingsMenu {
    #[strum(to_string = "Window Size")]
    WindowSize,
    #[strum(to_string = "Aspect Ratio")]
    AspectRatio,
}

impl VideoSettingsMenu {
    pub fn menu_event(&self) -> MenuEvent {
        MenuEvent::EditVideoSetting(*self)
    }

    pub fn setting_value(&self, video_settings: &Res<Persistent<VideoSettings>>) -> String {
        match self {
            Self::WindowSize => video_settings.window_size.to_string(),
            Self::AspectRatio => match video_settings.aspect_ratio {
                AspectRatio::Ar4_3 => "4:3".to_string(),
                AspectRatio::Ar16_10 => "16:10".to_string(),
                AspectRatio::Ar16_9 => "16:9".to_string(),
            },
        }
    }

    pub fn iter_events(
        video_settings: &Res<Persistent<VideoSettings>>,
    ) -> Vec<(String, MenuEvent)> {
        use strum::IntoEnumIterator;
        Self::iter()
            .map(|s| {
                (
                    format!("{}: {}", s, s.setting_value(video_settings)),
                    s.menu_event(),
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn iter_events_item(
        &self,
        video_settings: &Res<Persistent<VideoSettings>>,
    ) -> Vec<(String, MenuEvent)> {
        match self {
            Self::WindowSize => WindowSize::iter()
                .map(|s| {
                    (
                        if s == video_settings.window_size {
                            format!("[{s}]")
                        } else {
                            s.to_string()
                        },
                        MenuEvent::SaveVideoSetting(VideoSettingsValue::window_size(s)),
                    )
                })
                .collect::<Vec<_>>(),
            Self::AspectRatio => AspectRatio::iter()
                .map(|s| {
                    let label = match s {
                        AspectRatio::Ar4_3 => "4:3".to_string(),
                        AspectRatio::Ar16_10 => "16:10".to_string(),
                        AspectRatio::Ar16_9 => "16:9".to_string(),
                    };
                    (
                        if s == video_settings.aspect_ratio {
                            format!("[{label}]")
                        } else {
                            label
                        },
                        MenuEvent::SaveVideoSetting(VideoSettingsValue::aspect_ratio(s)),
                    )
                })
                .collect::<Vec<_>>(),
        }
    }
}
