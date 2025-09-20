//! Integration tests for Unhaunter game systems
//!
//! These tests verify that different components work together correctly.

use uncore::{
    components::board::{boardposition::BoardPosition, direction::Direction, position::Position},
    difficulty::{CurrentDifficulty, Difficulty},
    resources::summary_data::SummaryData,
    types::ghost::types::GhostType,
};

#[test]
fn test_difficulty_scoring_integration() {
    // Test that difficulty affects scoring correctly
    let ghost_types = vec![GhostType::BeanSidhe];

    let easy_difficulty =
        CurrentDifficulty(Difficulty::TutorialChapter1.create_difficulty_struct());
    let hard_difficulty = CurrentDifficulty(Difficulty::MasterChallenge.create_difficulty_struct());

    let mut easy_summary = SummaryData::new(ghost_types.clone(), easy_difficulty);
    let mut hard_summary = SummaryData::new(ghost_types, hard_difficulty);

    // Same mission parameters
    easy_summary.mission_successful = true;
    hard_summary.mission_successful = true;
    easy_summary.ghosts_unhaunted = 1;
    hard_summary.ghosts_unhaunted = 1;
    easy_summary.player_count = 1;
    hard_summary.player_count = 1;
    easy_summary.alive_count = 1;
    hard_summary.alive_count = 1;
    easy_summary.average_sanity = 100.0;
    hard_summary.average_sanity = 100.0;
    easy_summary.time_taken_secs = 300.0;
    hard_summary.time_taken_secs = 300.0;
    easy_summary.repellent_used_amt = 0;
    hard_summary.repellent_used_amt = 0;

    let easy_score = easy_summary.calculate_score();
    let hard_score = hard_summary.calculate_score();

    assert!(
        hard_score > easy_score,
        "Higher difficulty should yield higher score: {} vs {}",
        hard_score,
        easy_score
    );
}

#[test]
fn test_position_system_integration() {
    // Test that board positions and world positions work together
    let board_pos = BoardPosition::from_ndidx((5, 10, 0));
    let world_pos = board_pos.to_position();
    let center_pos = board_pos.to_position_centre();

    // Positions should be different but related
    assert_ne!(world_pos.x, center_pos.x);
    assert_ne!(world_pos.y, center_pos.y);

    // Both should be finite
    assert!(world_pos.x.is_finite());
    assert!(world_pos.y.is_finite());
    assert!(center_pos.x.is_finite());
    assert!(center_pos.y.is_finite());
}

#[test]
fn test_movement_calculation_integration() {
    // Test that position and direction systems work together
    let start_pos = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        global_z: 0.0,
    };

    let movement = Direction {
        dx: 3.0,
        dy: 4.0,
        dz: 0.0,
    };

    let normalized_movement = movement.normalised();
    let end_pos = normalized_movement.add_to_position(&start_pos);

    let distance_moved = start_pos.distance(&end_pos);

    // Should have moved unit distance (normalized)
    assert!(
        (distance_moved - 1.0).abs() < 0.001,
        "Normalized movement should be unit distance, got {}",
        distance_moved
    );
}

#[test]
fn test_ghost_difficulty_integration() {
    // Test that ghost systems work with difficulty systems
    for difficulty in [
        Difficulty::TutorialChapter1,
        Difficulty::StandardChallenge,
        Difficulty::MasterChallenge,
    ] {
        let config = difficulty.create_difficulty_struct();
        let ghosts = config.ghost_set.as_vec();

        assert!(
            !ghosts.is_empty(),
            "Difficulty {:?} should have ghosts",
            difficulty
        );

        for ghost in &ghosts {
            let evidences = ghost.evidences();
            assert_eq!(
                evidences.len(),
                5,
                "Ghost {:?} should have 5 evidences",
                ghost
            );
        }
    }
}

#[test]
fn test_complete_game_flow_simulation() {
    // Simulate a complete game flow
    let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];
    let difficulty = CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

    let mut summary = SummaryData::new(ghost_types, difficulty);

    // Simulate successful mission
    summary.mission_successful = true;
    summary.ghosts_unhaunted = 2;
    summary.player_count = 4;
    summary.alive_count = 3; // One player died
    summary.average_sanity = 75.0;
    summary.time_taken_secs = 900.0; // 15 minutes
    summary.repellent_used_amt = 2;

    let score = summary.calculate_score();

    assert!(score > 0, "Successful mission should have positive score");
    assert!(score < 10000, "Score should be reasonable");

    // Test that all components were involved
    assert!(summary.base_score > 0);
    assert!(summary.difficulty_multiplier > 0.0);
    assert_eq!(summary.full_score, score);
}
