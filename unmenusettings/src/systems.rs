use crate::components::{
    AudioSettingSelected, CustomNameInput, GameplaySettingSelected, MenuEvBack, MenuEvent,
    MenuItem, MenuSettingClassSelected, MenuType, ProfileSettingSelected, SaveAudioSetting,
    SaveGameplaySetting, SaveProfileSetting, SaveVideoSetting, SettingsMenu, SettingsState,
    SettingsStateTimer, TextInputField, VideoSettingSelected,
};
use crate::menu_ui::setup_ui_main_cat;
use crate::menus::{
    AudioSettingsMenu, GameplaySettingsMenu, MenuSettingsLevel1, ProfileSettingsMenu,
    ProfileSettingsValue, VideoSettingsMenu,
};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::colours::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::{MenuItemInteractive, MenuMouseTracker, MenuRoot};
use uncoremenu::systems::MenuItemClicked;
use uncoremenu::templates;
use unsettings::audio::AudioSettings;
use unsettings::game::GameplaySettings;
use unsettings::profile::ProfileSettings;
use unsettings::video::VideoSettings;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            item_highlight_system,
            menu_routing_system,
            menu_back_event,
            menu_settings_class_selected,
            menu_audio_setting_selected,
            menu_save_audio_setting,
            menu_gameplay_setting_selected,
            menu_save_gameplay_setting,
            menu_video_setting_selected,
            menu_save_video_setting,
            menu_profile_setting_selected,
            menu_save_profile_setting,
            menu_integration_system,
            handle_escape,
            custom_name_input_system,
            custom_name_text_input_system,
            update_custom_name_display_system,
            delete_custom_name_system,
        )
            .run_if(in_state(AppState::SettingsMenu)),
    )
    .add_event::<MenuEvent>()
    .add_event::<MenuEvBack>()
    .add_event::<MenuSettingClassSelected>()
    .add_event::<AudioSettingSelected>()
    .add_event::<SaveAudioSetting>()
    .add_event::<GameplaySettingSelected>()
    .add_event::<SaveGameplaySetting>()
    .add_event::<VideoSettingSelected>()
    .add_event::<SaveVideoSetting>()
    .add_event::<ProfileSettingSelected>()
    .add_event::<SaveProfileSetting>();
}

fn item_highlight_system(
    menu: Query<&SettingsMenu>,
    mut menu_items: Query<(&MenuItem, &mut TextColor)>,
) {
    let Ok(menu) = menu.single() else {
        return;
    }; // Assuming you have only one Menu component
    for (item, mut text_color) in &mut menu_items {
        let is_selected = item.idx == menu.selected_item_idx;
        let colour = if is_selected {
            MENU_ITEM_COLOR_ON
        } else {
            MENU_ITEM_COLOR_OFF
        };
        text_color.0 = colour;
    }
}

fn menu_routing_system(
    mut ev_menu: EventReader<MenuEvent>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut ev_class: EventWriter<MenuSettingClassSelected>,
    mut ev_audio_setting: EventWriter<AudioSettingSelected>,
    mut ev_save_audio_setting: EventWriter<SaveAudioSetting>,
    mut ev_game_setting: EventWriter<GameplaySettingSelected>,
    mut ev_save_game_setting: EventWriter<SaveGameplaySetting>,
    mut ev_video_setting: EventWriter<VideoSettingSelected>,
    mut ev_save_video_setting: EventWriter<SaveVideoSetting>,
    mut ev_profile_setting: EventWriter<ProfileSettingSelected>,
    mut ev_save_profile_setting: EventWriter<SaveProfileSetting>,
) {
    for ev in ev_menu.read() {
        match ev {
            MenuEvent::Back(menu_back) => {
                ev_back.write(menu_back.to_owned());
            }
            MenuEvent::None => {}
            MenuEvent::SettingClassSelected(menu_settings_level1) => {
                ev_class.write(MenuSettingClassSelected {
                    menu: menu_settings_level1.to_owned(),
                });
            }
            MenuEvent::EditAudioSetting(audio_settings_menu) => {
                ev_audio_setting.write(AudioSettingSelected {
                    setting: *audio_settings_menu,
                });
            }
            MenuEvent::SaveAudioSetting(setting_value) => {
                ev_save_audio_setting.write(SaveAudioSetting {
                    value: *setting_value,
                });
            }
            MenuEvent::EditGameplaySetting(gameplay_settings_menu) => {
                ev_game_setting.write(GameplaySettingSelected {
                    setting: *gameplay_settings_menu,
                });
            }
            MenuEvent::SaveGameplaySetting(setting_value) => {
                ev_save_game_setting.write(SaveGameplaySetting {
                    value: *setting_value,
                });
            }
            MenuEvent::EditVideoSetting(video_settings_menu) => {
                ev_video_setting.write(VideoSettingSelected {
                    setting: *video_settings_menu,
                });
            }
            MenuEvent::SaveVideoSetting(setting_value) => {
                ev_save_video_setting.write(SaveVideoSetting(*setting_value));
            }
            MenuEvent::EditProfileSetting(profile_settings_menu) => {
                ev_profile_setting.write(ProfileSettingSelected {
                    setting: *profile_settings_menu,
                });
            }
            MenuEvent::SaveProfileSetting(setting_value) => {
                ev_save_profile_setting.write(SaveProfileSetting {
                    value: setting_value.clone(),
                });
            }
            MenuEvent::StartCustomNameInput => {
                // This will be handled by the custom_name_input_system
            }
            MenuEvent::DeleteCustomName(_) => {
                // This will be handled by the delete_custom_name_system
            }
        }
    }
}

fn menu_back_event(
    mut events: EventReader<MenuEvBack>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
    settings_state: Res<State<SettingsState>>,
    mut ev_menu: EventWriter<MenuSettingClassSelected>,
    mut commands: Commands,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    for _ev in events.read() {
        match settings_state.get() {
            SettingsState::Lv1ClassSelection => {
                app_next_state.set(AppState::MainMenu);
                next_state.set(SettingsState::default());
            }
            SettingsState::Lv2List => {
                next_state.set(SettingsState::Lv1ClassSelection);
                // Redraw Main Menu:
                let menu_items = MenuSettingsLevel1::iter_events();
                setup_ui_main_cat(&mut commands, &handles, &qtui, "Settings", &menu_items);
            }
            SettingsState::Lv3ValueEdit(menu) => {
                ev_menu.write(MenuSettingClassSelected { menu: *menu });
            }
            SettingsState::CustomNameInput => {
                next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Profile));
            }
        }
    }
}

fn menu_settings_class_selected(
    mut commands: Commands,
    mut events: EventReader<MenuSettingClassSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    game_settings: Res<Persistent<GameplaySettings>>,
    video_settings: Res<Persistent<VideoSettings>>,
    profile_settings: Res<Persistent<ProfileSettings>>,
) {
    for ev in events.read() {
        warn!("Menu Setting Class Selected: {:?}", ev.menu);
        match ev.menu {
            MenuSettingsLevel1::Audio => {
                let menu_items = AudioSettingsMenu::iter_events(&audio_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Audio Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Gameplay => {
                let menu_items = GameplaySettingsMenu::iter_events(&game_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Gameplay Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Video => {
                let menu_items = VideoSettingsMenu::iter_events(&video_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Video Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Profile => {
                let menu_items = ProfileSettingsMenu::iter_events(&profile_settings);
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Profile Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
        }
    }
}

fn menu_audio_setting_selected(
    mut commands: Commands,
    mut events: EventReader<AudioSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    for ev in events.read() {
        warn!("Audio Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&audio_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        // Create new UI with uncoremenu templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                menu_type: MenuType::SettingEdit,
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &handles);

                // Logo
                templates::create_logo(parent, &handles);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Audio Settings",
                    ev.setting.to_string(),
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent, &handles, 0, // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

                content_area.insert(MenuRoot { selected_item: 0 });

                // Add a column container inside the content area for vertical layout
                content_area.with_children(|content| {
                    content
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|menu_list| {
                            let mut idx = 0;

                            // Add each menu item
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0, // First item selected by default
                                        &handles,
                                    )
                                    .insert(MenuItem::new(idx, event.clone()));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list, "Go Back", idx, false, &handles,
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[↑]/[↓]: Navigate • [Enter]: Select • [Esc]: Back".to_string()),
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Audio));
    }
}

fn menu_save_audio_setting(
    mut events: EventReader<SaveAudioSetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut audio_settings: ResMut<Persistent<AudioSettings>>,
) {
    use unsettings::audio::AudioSettingsValue as v;

    for ev in events.read() {
        warn!("Save Audio Setting: {:?}", ev.value);
        match ev.value {
            v::volume_master(audio_level) => {
                audio_settings.volume_master = audio_level;
            }
            v::volume_music(audio_level) => {
                audio_settings.volume_music = audio_level;
            }
            v::volume_effects(audio_level) => {
                audio_settings.volume_effects = audio_level;
            }
            v::volume_ambient(audio_level) => {
                audio_settings.volume_ambient = audio_level;
            }
            v::volume_voice_chat(audio_level) => {
                audio_settings.volume_voice_chat = audio_level;
            }
            v::sound_output(sound_output) => {
                audio_settings.sound_output = sound_output;
            }
            v::audio_positioning(audio_positioning) => {
                audio_settings.audio_positioning = audio_positioning;
            }
            v::feedback_delay(feedback_delay) => {
                audio_settings.feedback_delay = feedback_delay;
            }
            v::feedback_eq(feedback_eq) => {
                audio_settings.feedback_eq = feedback_eq;
            }
        }
        if let Err(e) = audio_settings.persist() {
            error!("Error persisting Audio Settings: {e:?}");
        }
        ev_back.write(MenuEvBack);
    }
}

fn menu_gameplay_setting_selected(
    mut commands: Commands,
    mut events: EventReader<GameplaySettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    for ev in events.read() {
        warn!("Gameplay Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&game_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        // Create new UI with uncoremenu templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                menu_type: MenuType::SettingEdit,
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &handles);

                // Logo

                templates::create_logo(parent, &handles);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Gameplay Settings",
                    ev.setting.to_string(),
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent, &handles, 0, // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

                content_area.insert(MenuRoot { selected_item: 0 });

                // Add a column container inside the content area for vertical layout
                content_area.with_children(|content| {
                    content
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|menu_list| {
                            let mut idx = 0;

                            // Add each menu item
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0, // First item selected by default
                                        &handles,
                                    )
                                    .insert(MenuItem::new(idx, event.clone()));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list, "Go Back", idx, false, &handles,
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[↑]/[↓]: Navigate • [Enter]: Select • [Esc]: Back".to_string()),
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Gameplay));
    }
}

fn menu_save_gameplay_setting(
    mut events: EventReader<SaveGameplaySetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut gameplay_settings: ResMut<Persistent<GameplaySettings>>,
) {
    use unsettings::game::GameplaySettingsValue as v;

    for ev in events.read() {
        warn!("Save Gameplay Setting: {:?}", ev.value);
        match ev.value {
            v::movement_style(movement_style) => {
                gameplay_settings.movement_style = movement_style;
            }
            v::camera_controls(camera_controls) => {
                gameplay_settings.camera_controls = camera_controls;
            }
            v::character_controls(character_controls) => {
                gameplay_settings.character_controls = character_controls;
            }
            v::dev_cheat_mode(dev_cheat_mode) => {
                gameplay_settings.dev_cheat_mode = dev_cheat_mode;
            }
        }
        if let Err(e) = gameplay_settings.persist() {
            error!("Error persisting Gameplay Settings: {e:?}");
        }
        ev_back.write(MenuEvBack);
    }
}

fn menu_integration_system(
    mut menu_clicks: EventReader<MenuItemClicked>,
    mut menu_events: EventWriter<MenuEvent>,
    menu_items: Query<(&MenuItem, &MenuItemInteractive)>,
    state_timer: Query<&SettingsStateTimer>,
) {
    // Define a small grace period to ignore events from previous state
    const GRACE_PERIOD_SECS: f32 = 0.1;

    // Get time since state entered
    if let Ok(timer) = state_timer.single() {
        let time_in_state = timer.state_entered_at.elapsed().as_secs_f32();

        // Ignore events that happened too soon after state transition
        if time_in_state < GRACE_PERIOD_SECS {
            menu_clicks.clear();
            return;
        }

        for click_event in menu_clicks.read() {
            if click_event.state != AppState::SettingsMenu {
                warn!(
                    "MenuItemClicked event received in state: {:?}",
                    click_event.state
                );
                continue;
            }
            warn!("Settings menu received click event: {:?}", click_event);
            let clicked_idx = click_event.pos;

            // Find the menu item with this index
            if let Some((menu_item, _)) = menu_items
                .iter()
                .find(|(_, interactive)| interactive.identifier == clicked_idx)
            {
                // Send the corresponding menu event
                menu_events.write(menu_item.on_activate.clone());
                warn!("Activating menu item: {:?}", menu_item.on_activate);
            } else {
                warn!("No menu item found with index {}", clicked_idx);
            }
        }
        menu_clicks.clear();
    }
}

/// Handles video setting selection events
fn menu_video_setting_selected(
    mut commands: Commands,
    mut events: EventReader<VideoSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    for ev in events.read() {
        warn!("Video Setting Selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&video_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        // Create new UI with uncoremenu templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                menu_type: MenuType::SettingEdit,
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &handles);

                // Logo
                templates::create_logo(parent, &handles);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Video Settings",
                    ev.setting.to_string(),
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent, &handles, 0, // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

                content_area.insert(MenuRoot { selected_item: 0 });

                // Add a column container inside the content area for vertical layout
                content_area.with_children(|content| {
                    content
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|menu_list| {
                            let mut idx = 0;

                            // Add each menu item
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0, // First item selected by default
                                        &handles,
                                    )
                                    .insert(MenuItem::new(idx, event.clone()));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list, "Go Back", idx, false, &handles,
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[↑]/[↓]: Navigate • [Enter]: Select • [Esc]: Back".to_string()),
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Video));
    }
}

/// Handles saving video settings
fn menu_save_video_setting(
    mut events: EventReader<SaveVideoSetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut video_settings: ResMut<Persistent<VideoSettings>>,
) {
    for ev in events.read() {
        warn!("Saving video setting: {:?}", ev.0);

        match ev.0 {
            unsettings::video::VideoSettingsValue::resolution(value) => {
                video_settings.resolution = value;
            }
            unsettings::video::VideoSettingsValue::aspect_ratio(value) => {
                video_settings.aspect_ratio = value;
            }
            unsettings::video::VideoSettingsValue::ui_zoom(value) => {
                video_settings.ui_zoom = value;
            }
            unsettings::video::VideoSettingsValue::refresh_rate(value) => {
                video_settings.refresh_rate = value;
            }
            unsettings::video::VideoSettingsValue::vsync(value) => {
                video_settings.vsync = value;
            }
        }

        if let Err(e) = video_settings.persist() {
            error!("Error persisting Video Settings: {e:?}");
        }
        ev_back.write(MenuEvBack);
    }
}

/// Handles profile setting selection
fn menu_profile_setting_selected(
    mut commands: Commands,
    mut events: EventReader<ProfileSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    profile_settings: Res<Persistent<ProfileSettings>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    for ev in events.read() {
        warn!("Profile setting selected: {:?}", ev.setting);

        let menu_items = ev.setting.iter_events_item(&profile_settings);

        // Clean up old UI
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }

        // Create new UI with uncoremenu templates
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            })
            .insert(SettingsMenu {
                menu_type: MenuType::SettingEdit,
                selected_item_idx: 0,
            })
            .with_children(|parent| {
                // Background
                templates::create_background(parent, &handles);

                // Logo
                templates::create_logo(parent, &handles);

                // Create breadcrumb navigation with title - show the full path
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Profile Settings",
                    ev.setting.to_string(),
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent, &handles, 0, // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

                content_area.insert(MenuRoot { selected_item: 0 });

                // Add a column container inside the content area for vertical layout
                content_area.with_children(|content| {
                    content
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|menu_list| {
                            let mut idx = 0;

                            // Add each menu item
                            for (item_text, event) in menu_items.iter() {
                                if !event.is_none() {
                                    templates::create_content_item(
                                        menu_list,
                                        item_text,
                                        idx,
                                        idx == 0, // First item selected by default
                                        &handles,
                                    )
                                    .insert(MenuItem::new(idx, event.clone()));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list, "Go Back", idx, false, &handles,
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[↑]/[↓]: Navigate • [Enter]: Select • [Esc]: Back".to_string()),
                );
            });

        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Profile));
    }
}

/// Handles saving profile settings
fn menu_save_profile_setting(
    mut events: EventReader<SaveProfileSetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut profile_settings: ResMut<Persistent<ProfileSettings>>,
) {
    for ev in events.read() {
        warn!("Saving profile setting: {:?}", ev.value);

        match &ev.value {
            crate::menus::ProfileSettingsValue::display_name(value) => {
                profile_settings.display_name = value.clone();
            }
            crate::menus::ProfileSettingsValue::colour(value) => {
                profile_settings.color = *value;
            }
        }

        if let Err(e) = profile_settings.persist() {
            error!("Error persisting Profile Settings: {e:?}");
        }
        ev_back.write(MenuEvBack);
    }
}

/// Handles the ESC key events from the core menu system
fn handle_escape(
    mut escape_events: EventReader<uncoremenu::systems::MenuEscapeEvent>,
    mut menu_events: EventWriter<MenuEvent>,
) {
    if !escape_events.is_empty() {
        // If ESC was pressed, send a Back event
        menu_events.write(MenuEvent::Back(MenuEvBack));
        escape_events.clear();
    }
}

/// Handles starting custom name input
fn custom_name_input_system(
    mut commands: Commands,
    mut events: EventReader<MenuEvent>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    for ev in events.read() {
        if matches!(ev, MenuEvent::StartCustomNameInput) {
            // Clean up old UI
            for e in qtui.iter() {
                commands.entity(e).despawn();
            }

            // Create new UI for custom name input
            commands
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                })
                .insert(SettingsMenu {
                    menu_type: MenuType::SettingEdit,
                    selected_item_idx: 0,
                })
                .with_children(|parent| {
                    // Background
                    templates::create_background(parent, &handles);

                    // Logo
                    templates::create_logo(parent, &handles);

                    // Create breadcrumb navigation
                    templates::create_breadcrumb_navigation(
                        parent,
                        &handles,
                        "Profile Settings > Display Name",
                        "Custom Name Input",
                    );

                    // Create content area
                    let mut content_area =
                        templates::create_selectable_content_area(parent, &handles, 0);

                    content_area.insert(MenuRoot { selected_item: 0 });

                    content_area.with_children(|content| {
                        content
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            })
                            .with_children(|input_container| {
                                // Instructions
                                templates::create_content_item(
                                    input_container,
                                    "Type your custom display name:",
                                    0,
                                    false,
                                    &handles,
                                );

                                // Text input field
                                input_container
                                    .spawn(Node {
                                        width: Val::Px(400.0),
                                        height: Val::Px(50.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    })
                                    .insert(BackgroundColor(Color::srgb(0.2, 0.2, 0.2)))
                                    .insert(TextInputField)
                                    .insert(CustomNameInput::default())
                                    .with_children(|text_container| {
                                        text_container
                                            .spawn(Text::new(""))
                                            .insert(TextFont {
                                                font: handles.fonts.titillium.w400_regular.clone(),
                                                font_size: 24.0,
                                                ..default()
                                            })
                                            .insert(TextColor(Color::WHITE));
                                    });

                                // Instructions
                                templates::create_content_item(
                                    input_container,
                                    "Press ENTER to save, ESC to cancel",
                                    1,
                                    false,
                                    &handles,
                                );
                            });
                    });

                    // Help text
                    templates::create_help_text(
                        parent,
                        &handles,
                        Some(
                            "Type your name and press [Enter] to save or [Escape] to cancel"
                                .to_string(),
                        ),
                    );
                });

            next_state.set(SettingsState::CustomNameInput);
        }
    }
}

/// Handles text input for custom name
fn custom_name_text_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut custom_input_query: Query<&mut CustomNameInput, With<TextInputField>>,
    mut profile_settings: ResMut<Persistent<ProfileSettings>>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut menu_events: EventWriter<MenuEvent>,
) {
    // Handle backspace
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        for mut custom_input in custom_input_query.iter_mut() {
            custom_input.current_text.pop();
        }
    }

    // Handle enter key - save the custom name
    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Some(custom_input) = custom_input_query.iter().next()
    {
        if !custom_input.current_text.trim().is_empty() {
            profile_settings.display_name = custom_input.current_text.trim().to_string();
            if let Err(e) = profile_settings.persist() {
                error!("Error persisting Profile Settings: {e:?}");
            }
            // Save the setting using the proper event
            menu_events.write(MenuEvent::SaveProfileSetting(
                ProfileSettingsValue::display_name(custom_input.current_text.trim().to_string()),
            ));
        }
        // Go back to profile settings
        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Profile));
    }

    // Handle escape key - cancel and go back
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Profile));
        menu_events.write(MenuEvent::EditProfileSetting(
            ProfileSettingsMenu::DisplayName,
        ));
    }

    // Handle character input for printable characters
    for key_code in keyboard_input.get_just_pressed() {
        if let Some(char) = key_code_to_char(*key_code, &keyboard_input) {
            for mut custom_input in custom_input_query.iter_mut() {
                // Only accept printable characters and limit name length to 20 characters
                if char.is_ascii_graphic() && custom_input.current_text.len() < 20 {
                    custom_input.current_text.push(char);
                }
            }
        }
    }
}

/// Updates the text display for custom name input
fn update_custom_name_display_system(
    custom_input_query: Query<(Entity, &CustomNameInput), With<TextInputField>>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    for (custom_input_entity, custom_input) in custom_input_query.iter() {
        // Find the text component that's a child of the custom input entity
        if let Ok(children) = children_query.get(custom_input_entity) {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = custom_input.current_text.clone();
                }
            }
        }
    }
}

/// Handles deletion of custom profile names
fn delete_custom_name_system(
    mut events: EventReader<MenuEvent>,
    mut profile_settings: ResMut<Persistent<ProfileSettings>>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut ev_profile_setting: EventWriter<ProfileSettingSelected>,
) {
    for event in events.read() {
        if let MenuEvent::DeleteCustomName(name_to_delete) = event {
            info!("Delete custom name requested: '{}'", name_to_delete);
            // Clear the display name if it matches the one being deleted
            if profile_settings.display_name == *name_to_delete {
                info!(
                    "Deleting custom name '{}' and resetting to default",
                    name_to_delete
                );
                // Reset to default "Player" name
                profile_settings.display_name = "Player".to_string();
                if let Err(e) = profile_settings.persist() {
                    error!("Error persisting Profile Settings after deletion: {e:?}");
                } else {
                    info!("Successfully deleted custom name and reset to default");
                }
            } else {
                warn!(
                    "Attempted to delete '{}' but current name is '{}'",
                    name_to_delete, profile_settings.display_name
                );
            }

            // Trigger a proper menu refresh by going back to level 2 and then to level 3
            next_state.set(SettingsState::Lv2List);
            // Then immediately trigger the profile setting selected event to rebuild the menu
            ev_profile_setting.write(ProfileSettingSelected {
                setting: ProfileSettingsMenu::DisplayName,
            });
            info!("Triggered menu refresh after delete");
        }
    }
}

/// Helper function to convert KeyCode to character, checking for shift modifier
fn key_code_to_char(key_code: KeyCode, keyboard_input: &ButtonInput<KeyCode>) -> Option<char> {
    let is_shift_pressed =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    match key_code {
        KeyCode::KeyA => Some(if is_shift_pressed { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if is_shift_pressed { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if is_shift_pressed { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if is_shift_pressed { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if is_shift_pressed { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if is_shift_pressed { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if is_shift_pressed { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if is_shift_pressed { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if is_shift_pressed { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if is_shift_pressed { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if is_shift_pressed { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if is_shift_pressed { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if is_shift_pressed { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if is_shift_pressed { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if is_shift_pressed { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if is_shift_pressed { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if is_shift_pressed { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if is_shift_pressed { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if is_shift_pressed { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if is_shift_pressed { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if is_shift_pressed { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if is_shift_pressed { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if is_shift_pressed { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if is_shift_pressed { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if is_shift_pressed { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if is_shift_pressed { 'Z' } else { 'z' }),
        KeyCode::Space => Some(' '),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        _ => None,
    }
}
