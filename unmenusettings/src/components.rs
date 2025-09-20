use bevy::prelude::*;
use bevy_platform::time::Instant;
use unsettings::{
    audio::AudioSettingsValue, game::GameplaySettingsValue, video::VideoSettingsValue,
};

use crate::menus::ProfileSettingsValue;

use crate::menus::{
    AudioSettingsMenu, GameplaySettingsMenu, MenuSettingsLevel1, ProfileSettingsMenu,
    VideoSettingsMenu,
};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MenuType {
    MainCategories,
    CategorySettingList,
    SettingEdit,
}

// Marker component for the main settings menu UI
#[derive(Component)]
pub struct SettingsMenu {
    pub menu_type: MenuType,
    pub selected_item_idx: usize,
}

#[derive(Component)]
pub struct SCamera;

#[derive(Component, Default)]
pub struct CustomNameInput {
    pub current_text: String,
}

#[derive(Component)]
pub struct TextInputField;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, States, Default)]
pub enum SettingsState {
    /// Selects which Setting file/category to edit in the UI (Audio, Video, etc)
    #[default]
    Lv1ClassSelection,
    /// Lists the settings available in the file for later editing (Volume, Control Type, etc)
    Lv2List,
    /// Allows the user to select a new value for the setting (10% volume, 50% volume, etc)
    Lv3ValueEdit(MenuSettingsLevel1),
    /// Allows the user to input a custom display name
    CustomNameInput,
}

#[derive(Component)]
pub struct SettingsStateTimer {
    pub state_entered_at: Instant,
}

#[derive(Component)]
pub struct MenuItem {
    pub idx: usize,
    pub on_activate: MenuEvent,
}

impl MenuItem {
    pub fn new(idx: usize, on_activate: MenuEvent) -> Self {
        MenuItem { idx, on_activate }
    }
}

#[derive(Event, Debug, Clone, Default)]
pub enum MenuEvent {
    SaveAudioSetting(AudioSettingsValue),
    EditAudioSetting(AudioSettingsMenu),
    SaveGameplaySetting(GameplaySettingsValue),
    EditGameplaySetting(GameplaySettingsMenu),
    SaveVideoSetting(VideoSettingsValue),
    EditVideoSetting(VideoSettingsMenu),
    SaveProfileSetting(ProfileSettingsValue),
    EditProfileSetting(ProfileSettingsMenu),
    StartCustomNameInput,
    DeleteCustomName(String),
    SettingClassSelected(MenuSettingsLevel1),
    Back(MenuEvBack),
    #[default]
    None,
}

impl MenuEvent {
    pub fn is_none(&self) -> bool {
        matches!(self, MenuEvent::None)
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MenuEvBack;

#[derive(Event, Debug, Clone, Copy)]
pub struct MenuSettingClassSelected {
    pub menu: MenuSettingsLevel1,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct AudioSettingSelected {
    pub setting: AudioSettingsMenu,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct VideoSettingSelected {
    pub setting: VideoSettingsMenu,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SaveVideoSetting(pub VideoSettingsValue);

#[derive(Event, Debug, Clone, Copy)]
pub struct SaveAudioSetting {
    pub value: AudioSettingsValue,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct GameplaySettingSelected {
    pub setting: GameplaySettingsMenu,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SaveGameplaySetting {
    pub value: GameplaySettingsValue,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ProfileSettingSelected {
    pub setting: ProfileSettingsMenu,
}

#[derive(Event, Debug, Clone)]
pub struct SaveProfileSetting {
    pub value: ProfileSettingsValue,
}
