//! Player Light Level System
//!
//! This module handles calculating and updating the light level at the player's position.
//! The LightLevel component is used by various systems to determine lighting conditions
//! for gameplay mechanics like sanity effects and walkie talkie hints.

use bevy::prelude::*;
use uncore::{
    components::{
        board::position::Position,
        light::LightLevel,
        player_sprite::PlayerSprite,
    },
    resources::board_data::BoardData,
    states::{AppState, GameState},
};

/// System that calculates and updates the light level at the player's position
pub fn update_player_light_level_system(
    mut player_query: Query<(&Position, &mut LightLevel), With<PlayerSprite>>,
    board_data: Res<BoardData>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
) {
    // Only run during gameplay
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        return;
    }

    for (position, mut light_level) in player_query.iter_mut() {
        let board_pos = position.to_board_position();
        let idx = board_pos.ndidx();
        
        // Get the light level from the board's light field
        if let Some(light_field_data) = board_data.light_field.get(idx) {
            light_level.lux = light_field_data.lux;
        } else {
            // Fallback to 0 if no light data available
            light_level.lux = 0.0;
        }
    }
}

/// System that adds the LightLevel component to player entities that don't have it
pub fn ensure_player_light_level_system(
    mut commands: Commands,
    player_query: Query<Entity, (With<PlayerSprite>, Without<LightLevel>)>,
) {
    for entity in player_query.iter() {
        commands.entity(entity).insert(LightLevel::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            ensure_player_light_level_system,
            update_player_light_level_system,
        )
            .chain()
            .run_if(in_state(AppState::InGame).and(in_state(GameState::None))),
    );
}


