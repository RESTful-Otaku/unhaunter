use bevy::prelude::*;
use uncore::{
    components::{game_config::GameConfig, player_sprite::PlayerSprite},
    input::{ActionState, PlayerAction},
    resources::looking_gear::LookingGear,
    states::AppState,
};

fn system_update_looking_gear(
    actions: Res<ActionState>,
    mut looking_gear: ResMut<LookingGear>,
    gc: Res<GameConfig>,
    players: Query<&PlayerSprite>,
) {
    let Some(_player_sprite) = players.iter().find(|player| player.id == gc.player_id) else {
        return;
    };
    if actions.just_pressed(PlayerAction::LookLeftHandToggle) {
        looking_gear.toggle();
    }

    looking_gear.held = actions.pressed(PlayerAction::LookLeftHandHold);
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<LookingGear>().add_systems(
        Update,
        system_update_looking_gear.run_if(in_state(AppState::InGame)),
    );
}
