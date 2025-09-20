//! Simple integration tests that don't depend on the main binary
//!
//! These tests verify that the core modules work together correctly
//! without requiring the full application setup.

use uncore::{
    difficulty::{CurrentDifficulty, Difficulty},
    resources::summary_data::SummaryData,
    types::ghost::types::GhostType,
};

/// Test that difficulty system integrates properly with scoring system
#[test]
fn test_difficulty_scoring_integration() {
    let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];

    // Test different difficulties produce different scores
    let difficulties = [
        Difficulty::TutorialChapter1,
        Difficulty::StandardChallenge,
        Difficulty::MasterChallenge,
    ];

    let mut scores = Vec::new();

    for difficulty in difficulties.iter() {
        let current_difficulty = CurrentDifficulty(difficulty.create_difficulty_struct());
        let mut summary = SummaryData::new(ghost_types.clone(), current_difficulty);

        // Set up a successful mission
        summary.mission_successful = true;
        summary.ghosts_unhaunted = 2;
        summary.player_count = 1;
        summary.alive_count = 1;
        summary.average_sanity = 80.0;
        summary.time_taken_secs = 300.0;
        summary.repellent_used_amt = 0;

        let score = summary.calculate_score();
        scores.push((*difficulty, score));

        // Each difficulty should produce a valid score
        assert!(
            score >= 0,
            "Score should be non-negative for {:?}",
            difficulty
        );
    }

    // Verify score progression makes sense
    assert!(
        scores[2].1 > scores[0].1,
        "Master difficulty should score higher than tutorial"
    );
}

/// Test that ghost system integrates with difficulty system
#[test]
fn test_ghost_difficulty_integration() {
    for difficulty in enum_iterator::all::<Difficulty>() {
        let config = difficulty.create_difficulty_struct();

        // Verify ghost set is valid for this difficulty
        match &config.ghost_set {
            uncore::types::ghost::definitions::GhostSet::All => {
                // All ghosts should be available
                let all_ghosts: Vec<GhostType> = enum_iterator::all::<GhostType>().collect();
                assert!(
                    !all_ghosts.is_empty(),
                    "Should have ghosts available for difficulty {:?}",
                    difficulty
                );
            }
            uncore::types::ghost::definitions::GhostSet::Twenty
            | uncore::types::ghost::definitions::GhostSet::TmpEMF
            | uncore::types::ghost::definitions::GhostSet::TmpEMFUVOrbs
            | uncore::types::ghost::definitions::GhostSet::TmpEMFUVOrbsEVPCPM => {
                let ghosts = config.ghost_set.as_vec();
                // Subset should not be empty
                assert!(
                    !ghosts.is_empty(),
                    "Ghost subset should not be empty for difficulty {:?}",
                    difficulty
                );

                // All ghosts in subset should be valid
                for ghost in &ghosts {
                    let evidences = ghost.evidences();
                    assert_eq!(
                        evidences.len(),
                        5,
                        "Ghost {:?} should have exactly 5 evidences",
                        ghost
                    );
                }
            }
        }
    }
}

/// Test complete mission workflow
#[test]
fn test_complete_mission_workflow() {
    // Set up a complete mission scenario
    let difficulty = Difficulty::StandardChallenge;
    let ghost_types = vec![GhostType::BeanSidhe];

    // Create difficulty configuration
    let difficulty_config = difficulty.create_difficulty_struct();
    let current_difficulty = CurrentDifficulty(difficulty_config);

    // Verify difficulty is properly configured
    assert!(!current_difficulty.0.difficulty_name.is_empty());
    assert!(current_difficulty.0.difficulty_score_multiplier > 0.0);

    // Create mission summary
    let mut summary = SummaryData::new(ghost_types.clone(), current_difficulty);

    // Simulate successful mission
    summary.mission_successful = true;
    summary.ghosts_unhaunted = ghost_types.len() as u32;
    summary.player_count = 1;
    summary.alive_count = 1;
    summary.average_sanity = 85.0;
    summary.time_taken_secs = 420.0; // 7 minutes
    summary.repellent_used_amt = 1;

    // Calculate final score
    let final_score = summary.calculate_score();

    // Verify complete workflow produced valid results
    assert!(
        final_score > 0,
        "Successful mission should have positive score"
    );
    assert!(summary.base_score > 0, "Base score should be calculated");
    assert!(
        summary.difficulty_multiplier > 0.0,
        "Difficulty multiplier should be applied"
    );
    assert_eq!(
        summary.full_score, final_score,
        "Full score should match calculated score"
    );

    // Verify score components are reasonable
    assert!(
        summary.base_score <= summary.full_score,
        "Full score should be at least base score after multiplier"
    );
}

/// Test system consistency across multiple operations
#[test]
fn test_system_consistency() {
    // Test that repeated operations produce consistent results
    let difficulty = Difficulty::StandardChallenge;

    for _ in 0..10 {
        let config1 = difficulty.create_difficulty_struct();
        let config2 = difficulty.create_difficulty_struct();

        // Should produce identical configurations
        assert_eq!(config1.difficulty_name, config2.difficulty_name);
        assert_eq!(
            config1.difficulty_score_multiplier,
            config2.difficulty_score_multiplier
        );
        assert_eq!(config1.ghost_speed, config2.ghost_speed);
        assert_eq!(config1.sanity_drain_rate, config2.sanity_drain_rate);
    }
}

/// Test performance of integrated systems
#[test]
fn test_integrated_system_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Simulate multiple mission calculations
    for difficulty_idx in 0..3 {
        let difficulties = [
            Difficulty::TutorialChapter1,
            Difficulty::StandardChallenge,
            Difficulty::MasterChallenge,
        ];

        let difficulty = difficulties[difficulty_idx];
        let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];
        let current_difficulty = CurrentDifficulty(difficulty.create_difficulty_struct());

        for _ in 0..100 {
            let mut summary = SummaryData::new(ghost_types.clone(), current_difficulty.clone());
            summary.mission_successful = true;
            summary.ghosts_unhaunted = 2;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = 0;

            let _score = summary.calculate_score();
        }
    }

    let duration = start.elapsed();

    // Should complete 300 integrated operations quickly
    assert!(
        duration.as_millis() < 100,
        "Integrated system operations too slow: {}ms",
        duration.as_millis()
    );
}
