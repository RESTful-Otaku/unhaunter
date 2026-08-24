use crate::components::{
    AudioSettingSelected, ControlSettingSelected, GameplaySettingSelected, MenuEvBack, MenuEvent,
    MenuItem, MenuSettingClassSelected, MenuType, RebindRequest, SaveAudioSetting,
    SaveControlSetting, SaveGameplaySetting, SettingsMenu, SettingsState, SettingsStateTimer,
};
use crate::menu_ui::setup_ui_main_cat;
use crate::menus::{AudioSettingsMenu, GameplaySettingsMenu, MenuSettingsLevel1};
use crate::menus_bindings::{BindDevice, ControlSettingsMenu, StickSettingsMenu, rebind_list_rows};
use bevy::input::gamepad::GamepadButtonChangedEvent;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use strum::IntoEnumIterator;
use uncore::colors::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use uncore::input::GamepadStatus;
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::{MenuItemInteractive, MenuMouseTracker, MenuRoot};
use uncoremenu::systems::MenuItemClicked;
use uncoremenu::templates;
use unsettings::audio::AudioSettings;
use unsettings::bindings::{ControlBindings, ControlSettingValue, InputDeviceMode};
use unsettings::game::GameplaySettings;

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
            menu_control_setting_selected,
            menu_save_control_setting,
            menu_rebind_request,
            rebind_capture_system,
            menu_integration_system,
            handle_escape,
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
    .add_event::<ControlSettingSelected>()
    .add_event::<SaveControlSetting>()
    .add_event::<RebindRequest>();
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
        let color = if is_selected {
            MENU_ITEM_COLOR_ON
        } else {
            MENU_ITEM_COLOR_OFF
        };
        text_color.0 = color;
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
    mut ev_control_setting: EventWriter<ControlSettingSelected>,
    mut ev_save_control_setting: EventWriter<SaveControlSetting>,
    mut ev_rebind_request: EventWriter<RebindRequest>,
) {
    for ev in ev_menu.read() {
        match ev {
            MenuEvent::Back(menu_back) => {
                ev_back.write(menu_back.to_owned());
            }
            MenuEvent::None | MenuEvent::BindingInfo => {}
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
            MenuEvent::EditControlSetting(control_settings_menu) => {
                ev_control_setting.write(ControlSettingSelected {
                    setting: *control_settings_menu,
                });
            }
            MenuEvent::SaveControlSetting(value) => {
                ev_save_control_setting.write(SaveControlSetting { value: *value });
            }
            MenuEvent::RebindRequest(device, action) => {
                ev_rebind_request.write(RebindRequest {
                    device: *device,
                    action: *action,
                });
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
    control_bindings: Res<Persistent<ControlBindings>>,
    gamepad_status: Res<GamepadStatus>,
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
            // Fallback safety net: a Back during capture returns to the
            // Controls category page (normally handled by the capture system).
            SettingsState::RebindCapture { .. } => {
                let menu_items =
                    ControlSettingsMenu::iter_events(&control_bindings, &gamepad_status.summary());
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Controls Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
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
    control_bindings: Res<Persistent<ControlBindings>>,
    gamepad_status: Res<GamepadStatus>,
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
            MenuSettingsLevel1::Controls => {
                let menu_items =
                    ControlSettingsMenu::iter_events(&control_bindings, &gamepad_status.summary());
                setup_ui_main_cat(
                    &mut commands,
                    &handles,
                    &qtui,
                    "Controls Settings",
                    &menu_items,
                );
                next_state.set(SettingsState::Lv2List);
            }
            MenuSettingsLevel1::Video => todo!(),
            MenuSettingsLevel1::Profile => todo!(),
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
                    ev.setting.to_string()
                );

                // Create content area for settings items
                let mut content_area = templates::create_selectable_content_area(
                    parent,
                    &handles,
                    0 // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

                content_area.insert(MenuRoot {
                    selected_item: 0,
                });

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
                                        &handles
                                    )
                                    .insert(MenuItem::new(idx, *event));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list,
                                "Go Back",
                                idx,
                                false,
                                &handles
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
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
                    parent,
                    &handles,
                    0 // Initial selection
                );

                // Add mouse tracker to prevent unwanted initial hover selection
                content_area.insert(MenuMouseTracker::default());

                content_area.insert(MenuRoot {
                    selected_item: 0,
                });

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
                                        &handles
                                    )
                                    .insert(MenuItem::new(idx, *event));
                                    idx += 1;
                                }
                            }

                            // Add "Go Back" option
                            templates::create_content_item(
                                menu_list,
                                "Go Back",
                                idx,
                                false,
                                &handles
                            )
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                        });
                });

                // Help text
                templates::create_help_text(
                    parent,
                    &handles,
                    Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
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
                menu_events.write(menu_item.on_activate);
                warn!("Activating menu item: {:?}", menu_item.on_activate);
            } else {
                warn!("No menu item found with index {}", clicked_idx);
            }
        }
        menu_clicks.clear();
    }
}

/// Handles the ESC key events from the core menu system
fn handle_escape(
    settings_state: Res<State<SettingsState>>,
    mut escape_events: EventReader<uncoremenu::systems::MenuEscapeEvent>,
    mut menu_events: EventWriter<MenuEvent>,
) {
    // While capturing a new binding the capture system handles ESC itself
    // (cancel and redraw), so it must not bubble up as a generic Back.
    if matches!(settings_state.get(), SettingsState::RebindCapture { .. }) {
        escape_events.clear();
        return;
    }
    if !escape_events.is_empty() {
        // If ESC was pressed, send a Back event
        menu_events.write(MenuEvent::Back(MenuEvBack));
        escape_events.clear();
    }
}

/// Spawns a level-3 style edit page with arbitrary selectable rows.
fn spawn_edit_page(
    commands: &mut Commands,
    handles: &GameAssets,
    qtui: &Query<Entity, With<SettingsMenu>>,
    title: &str,
    breadcrumb: &str,
    help_text: Option<String>,
    menu_items: Vec<(String, MenuEvent)>,
) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }
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
            templates::create_background(parent, handles);
            templates::create_logo(parent, handles);
            templates::create_breadcrumb_navigation(parent, handles, title, breadcrumb);

            let mut content_area = templates::create_selectable_content_area(parent, handles, 0);
            content_area.insert(MenuMouseTracker::default());
            content_area.insert(MenuRoot { selected_item: 0 });

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
                        for (item_text, event) in menu_items.iter() {
                            if !event.is_none() {
                                templates::create_content_item(
                                    menu_list,
                                    item_text,
                                    idx,
                                    idx == 0,
                                    handles,
                                )
                                .insert(MenuItem::new(idx, *event));
                                idx += 1;
                            }
                        }
                        templates::create_content_item(menu_list, "Go Back", idx, false, handles)
                            .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                    });
            });

            templates::create_help_text(parent, handles, help_text);
        });
}

/// Builds the level-3 rows shown when a Controls entry is activated.
fn control_page_rows(
    setting: ControlSettingsMenu,
    bindings: &Persistent<ControlBindings>,
) -> (String, Option<String>, Vec<(String, MenuEvent)>) {
    match setting {
        ControlSettingsMenu::DeviceMode => (
            "Input Devices".to_string(),
            Some("Auto lets keyboard and gamepads work together. Gamepad-only falls back to the keyboard when no pad is connected.".to_string()),
            InputDeviceMode::iter()
                .map(|m| {
                    let label = if m == bindings.device_mode {
                        format!("[{m}]")
                    } else {
                        m.to_string()
                    };
                    (
                        label,
                        MenuEvent::SaveControlSetting(ControlSettingValue::DeviceMode(m)),
                    )
                })
                .collect(),
        ),
        ControlSettingsMenu::StickSettings => (
            StickSettingsMenu::MoveDeadzone.to_string(),
            Some("Fine-tune analog sticks. Deadzones ignore small drift; sensitivity scales deflection; the curve shapes precision near center.".to_string()),
            StickSettingsMenu::build_rows(bindings),
        ),
        ControlSettingsMenu::KeyboardBindings => (
            "Keyboard Bindings".to_string(),
            Some("Select an action to assign a new key. Assigning a key already in use moves it here.".to_string()),
            rebind_list_rows(BindDevice::Keyboard, bindings),
        ),
        ControlSettingsMenu::GamepadBindings => (
            "Gamepad Bindings".to_string(),
            Some("Select an action to assign a new button. Assigning a button already in use moves it here.".to_string()),
            rebind_list_rows(BindDevice::Gamepad, bindings),
        ),
        ControlSettingsMenu::RunMode => (
            "Run Mode".to_string(),
            None,
            [false, true]
                .into_iter()
                .map(|toggle| {
                    let label = if toggle { "Toggle" } else { "Hold" }.to_string();
                    let label = if toggle == bindings.run_is_toggle {
                        format!("[{label}]")
                    } else {
                        label
                    };
                    (
                        label,
                        MenuEvent::SaveControlSetting(ControlSettingValue::RunIsToggle(toggle)),
                    )
                })
                .collect(),
        ),
        ControlSettingsMenu::Rumble => (
            "Rumble Feedback".to_string(),
            None,
            [false, true]
                .into_iter()
                .map(|on| {
                    let label = if on { "On" } else { "Off" }.to_string();
                    let label = if on == bindings.rumble_enabled {
                        format!("[{label}]")
                    } else {
                        label
                    };
                    (
                        label,
                        MenuEvent::SaveControlSetting(ControlSettingValue::RumbleEnabled(on)),
                    )
                })
                .collect(),
        ),
        // Display-only entries never route here.
        ControlSettingsMenu::ConnectedPads => ("Connected Gamepads".to_string(), None, vec![]),
        ControlSettingsMenu::ResetKeyboard | ControlSettingsMenu::ResetGamepad => {
            unreachable!("reset entries save directly")
        }
    }
}

fn menu_control_setting_selected(
    mut commands: Commands,
    mut events: EventReader<ControlSettingSelected>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    control_bindings: Res<Persistent<ControlBindings>>,
) {
    for ev in events.read() {
        warn!("Control Setting Selected: {:?}", ev.setting);
        let (breadcrumb, help, rows) = control_page_rows(ev.setting, &control_bindings);
        spawn_edit_page(
            &mut commands,
            &handles,
            &qtui,
            "Controls Settings",
            &format!("Controls > {breadcrumb}"),
            help,
            rows,
        );
        next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Controls));
    }
}

fn menu_save_control_setting(
    mut events: EventReader<SaveControlSetting>,
    mut ev_back: EventWriter<MenuEvBack>,
    mut control_bindings: ResMut<Persistent<ControlBindings>>,
) {
    for ev in events.read() {
        warn!("Save Control Setting: {:?}", ev.value);
        control_bindings.apply_setting(ev.value);
        if let Err(e) = control_bindings.persist() {
            error!("Error persisting Control Bindings: {e:?}");
        }
        ev_back.write(MenuEvBack);
    }
}

fn menu_rebind_request(
    mut commands: Commands,
    mut events: EventReader<RebindRequest>,
    mut next_state: ResMut<NextState<SettingsState>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    for ev in events.read() {
        warn!("Rebind Request: {:?} {:?}", ev.device, ev.action);
        for e in qtui.iter() {
            commands.entity(e).despawn();
        }
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
                templates::create_background(parent, &handles);
                templates::create_breadcrumb_navigation(
                    parent,
                    &handles,
                    "Controls Settings",
                    match ev.device {
                        BindDevice::Keyboard => "Controls > Keyboard Bindings > Rebind",
                        BindDevice::Gamepad => "Controls > Gamepad Bindings > Rebind",
                    },
                );
                let prompt = format!(
                    "Press a {} for \"{}\"",
                    match ev.device {
                        BindDevice::Keyboard => "key",
                        BindDevice::Gamepad => "button",
                    },
                    ev.action.label()
                );
                templates::create_help_text(
                    parent,
                    &handles,
                    Some(format!("{prompt}\nPress [Escape] to cancel")),
                );
            });
        next_state.set(SettingsState::RebindCapture {
            device: ev.device,
            action: ev.action,
        });
    }
}

/// Listens for raw input while waiting for a rebind, applies it, persists and
/// returns to the binding list.
#[allow(clippy::too_many_arguments)]
fn rebind_capture_system(
    mut commands: Commands,
    settings_state: Res<State<SettingsState>>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut key_input: ResMut<ButtonInput<KeyCode>>,
    mut gamepad_button_events: EventReader<GamepadButtonChangedEvent>,
    mut control_bindings: ResMut<Persistent<ControlBindings>>,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
    mut frames_since_enter: Local<u32>,
) {
    let Ok((device, action)) = (match settings_state.get() {
        SettingsState::RebindCapture { device, action } => Ok((*device, *action)),
        _ => Err(()),
    }) else {
        *frames_since_enter = 0;
        return;
    };
    *frames_since_enter = frames_since_enter.saturating_add(1);
    // Grace period so the Enter/click that opened this page is not captured.
    if *frames_since_enter < 2 {
        return;
    }

    // Keyboard capture. Escape cancels.
    for key in key_input.get_just_pressed().copied().collect::<Vec<_>>() {
        key_input.clear_just_pressed(key);
        if key == KeyCode::Escape {
            cancel_rebind(
                &mut commands,
                &handles,
                &qtui,
                device,
                action,
                &control_bindings,
                &mut next_state,
            );
            return;
        }
        match device {
            BindDevice::Keyboard => {
                control_bindings.set_key(action, key);
                finish_rebind(
                    &mut commands,
                    &handles,
                    &qtui,
                    device,
                    action,
                    &control_bindings,
                    &mut next_state,
                );
            }
            BindDevice::Gamepad => continue,
        }
        return;
    }

    // Gamepad capture.
    if matches!(device, BindDevice::Keyboard) {
        return;
    }
    for ev in gamepad_button_events.read() {
        if ev.state != bevy::input::ButtonState::Pressed || ev.value < 0.5 {
            continue;
        }
        control_bindings.set_button(action, ev.button);
        finish_rebind(
            &mut commands,
            &handles,
            &qtui,
            device,
            action,
            &control_bindings,
            &mut next_state,
        );
        return;
    }
}

fn cancel_rebind(
    commands: &mut Commands,
    handles: &GameAssets,
    qtui: &Query<Entity, With<SettingsMenu>>,
    device: BindDevice,
    action: unsettings::bindings::PlayerAction,
    control_bindings: &Persistent<ControlBindings>,
    next_state: &mut NextState<SettingsState>,
) {
    redraw_rebind_list(
        commands,
        handles,
        qtui,
        device,
        action,
        control_bindings,
        next_state,
    );
}

fn finish_rebind(
    commands: &mut Commands,
    handles: &GameAssets,
    qtui: &Query<Entity, With<SettingsMenu>>,
    device: BindDevice,
    action: unsettings::bindings::PlayerAction,
    control_bindings: &Persistent<ControlBindings>,
    next_state: &mut NextState<SettingsState>,
) {
    if let Err(e) = control_bindings.persist() {
        error!("Error persisting Control Bindings: {e:?}");
    }
    redraw_rebind_list(
        commands,
        handles,
        qtui,
        device,
        action,
        control_bindings,
        next_state,
    );
}

fn redraw_rebind_list(
    commands: &mut Commands,
    handles: &GameAssets,
    qtui: &Query<Entity, With<SettingsMenu>>,
    device: BindDevice,
    action: unsettings::bindings::PlayerAction,
    control_bindings: &Persistent<ControlBindings>,
    next_state: &mut NextState<SettingsState>,
) {
    info!(
        "Rebound {} on {}: done",
        action.label(),
        match device {
            BindDevice::Keyboard => "keyboard",
            BindDevice::Gamepad => "gamepad",
        }
    );
    let _ = action;
    let rows = rebind_list_rows(device, control_bindings);
    spawn_edit_page(
        commands,
        handles,
        qtui,
        "Controls Settings",
        &match device {
            BindDevice::Keyboard => "Controls > Keyboard Bindings".to_string(),
            BindDevice::Gamepad => "Controls > Gamepad Bindings".to_string(),
        },
        Some("Select an action to assign a new input. Press [Escape] to go back.".to_string()),
        rows,
    );
    next_state.set(SettingsState::Lv3ValueEdit(MenuSettingsLevel1::Controls));
}
