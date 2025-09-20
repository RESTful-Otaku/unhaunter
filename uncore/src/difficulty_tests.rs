//! Comprehensive unit tests for the difficulty system
//!
//! These tests verify the correctness and reliability of difficulty calculations,
//! gear restrictions, and ghost set configurations across all difficulty levels.

#[cfg(test)]
mod tests {
    use super::super::difficulty::Difficulty;
    // Gear kinds removed as they're not used in current tests
    use crate::types::ghost::definitions::GhostSet;
    use enum_iterator::all;
    use proptest::prelude::*;
    use quickcheck::{TestResult, quickcheck};
    use rstest::*;

    // Test fixtures for common difficulty scenarios
    #[fixture]
    fn tutorial_difficulty() -> Difficulty {
        Difficulty::TutorialChapter1
    }

    #[fixture]
    fn standard_difficulty() -> Difficulty {
        Difficulty::StandardChallenge
    }

    #[fixture]
    fn master_difficulty() -> Difficulty {
        Difficulty::MasterChallenge
    }

    /// Test that all difficulty levels have valid configurations
    #[test]
    fn test_all_difficulties_have_valid_configs() {
        for difficulty in all::<Difficulty>() {
            let config = difficulty.create_difficulty_struct();

            // All difficulties must have non-empty names
            assert!(
                !config.difficulty_name.is_empty(),
                "Difficulty {:?} has empty name",
                difficulty
            );

            // Score multiplier must be positive
            assert!(
                config.difficulty_score_multiplier > 0.0,
                "Difficulty {:?} has invalid score multiplier: {}",
                difficulty,
                config.difficulty_score_multiplier
            );

            // Ghost speed must be reasonable
            assert!(
                config.ghost_speed > 0.0,
                "Difficulty {:?} has invalid ghost speed: {}",
                difficulty,
                config.ghost_speed
            );

            // Sanity drain rate must be non-negative
            assert!(
                config.sanity_drain_rate >= 0.0,
                "Difficulty {:?} has negative sanity drain rate: {}",
                difficulty,
                config.sanity_drain_rate
            );
        }
    }

    /// Test tutorial difficulty progression
    #[rstest]
    #[case(Difficulty::TutorialChapter1, Difficulty::TutorialChapter2)]
    #[case(Difficulty::TutorialChapter2, Difficulty::TutorialChapter3)]
    #[case(Difficulty::TutorialChapter3, Difficulty::TutorialChapter4)]
    #[case(Difficulty::TutorialChapter4, Difficulty::TutorialChapter5)]
    fn test_tutorial_progression_increases_difficulty(
        #[case] easier: Difficulty,
        #[case] harder: Difficulty,
    ) {
        let easier_config = easier.create_difficulty_struct();
        let harder_config = harder.create_difficulty_struct();

        // Later tutorials should have higher score multipliers
        assert!(
            harder_config.difficulty_score_multiplier >= easier_config.difficulty_score_multiplier,
            "Tutorial progression broken: {:?} (x{}) should be harder than {:?} (x{})",
            harder,
            harder_config.difficulty_score_multiplier,
            easier,
            easier_config.difficulty_score_multiplier
        );
    }

    /// Test challenge difficulty progression
    #[rstest]
    #[case(Difficulty::StandardChallenge, Difficulty::HardChallenge)]
    #[case(Difficulty::HardChallenge, Difficulty::ExpertChallenge)]
    #[case(Difficulty::ExpertChallenge, Difficulty::MasterChallenge)]
    fn test_challenge_progression_increases_difficulty(
        #[case] easier: Difficulty,
        #[case] harder: Difficulty,
    ) {
        let easier_config = easier.create_difficulty_struct();
        let harder_config = harder.create_difficulty_struct();

        // Higher challenges should have higher score multipliers
        assert!(
            harder_config.difficulty_score_multiplier > easier_config.difficulty_score_multiplier,
            "Challenge progression broken: {:?} (x{}) should be harder than {:?} (x{})",
            harder,
            harder_config.difficulty_score_multiplier,
            easier,
            easier_config.difficulty_score_multiplier
        );
    }

    /// Test difficulty parameters are reasonable
    #[test]
    fn test_difficulty_parameters_reasonable() {
        let tutorial = Difficulty::TutorialChapter1.create_difficulty_struct();
        let master = Difficulty::MasterChallenge.create_difficulty_struct();

        // Tutorial should be easier than master
        assert!(
            tutorial.difficulty_score_multiplier <= master.difficulty_score_multiplier,
            "Tutorial should have lower or equal score multiplier than master"
        );

        // Both should have reasonable ghost speeds
        assert!(
            tutorial.ghost_speed > 0.0,
            "Tutorial ghost speed should be positive"
        );
        assert!(
            master.ghost_speed > 0.0,
            "Master ghost speed should be positive"
        );

        // Sanity drain should be reasonable
        assert!(
            tutorial.sanity_drain_rate >= 0.0,
            "Tutorial sanity drain should be non-negative"
        );
        assert!(
            master.sanity_drain_rate >= 0.0,
            "Master sanity drain should be non-negative"
        );
    }

    /// Test ghost set configurations
    #[test]
    fn test_ghost_set_configurations() {
        for difficulty in all::<Difficulty>() {
            let config = difficulty.create_difficulty_struct();

            // Ghost set should be valid
            match &config.ghost_set {
                GhostSet::All => {} // All ghosts is valid
                GhostSet::Twenty
                | GhostSet::TmpEMF
                | GhostSet::TmpEMFUVOrbs
                | GhostSet::TmpEMFUVOrbsEVPCPM => {
                    let ghosts = config.ghost_set.as_vec();
                    assert!(
                        !ghosts.is_empty(),
                        "Difficulty {:?} has empty ghost set",
                        difficulty
                    );
                }
            }
        }
    }

    quickcheck! {
        fn prop_score_multiplier_consistency(difficulty_idx: u8) -> TestResult {
            let difficulties: Vec<Difficulty> = all::<Difficulty>().collect();
            if difficulty_idx as usize >= difficulties.len() {
                return TestResult::discard();
            }

            let difficulty = difficulties[difficulty_idx as usize];
            let config1 = difficulty.create_difficulty_struct();
            let config2 = difficulty.create_difficulty_struct();

            // Same difficulty should always produce same multiplier
            TestResult::from_bool(
                config1.difficulty_score_multiplier == config2.difficulty_score_multiplier
            )
        }
    }

    proptest! {
        #[test]
        fn prop_ghost_speed_in_reasonable_range(difficulty_idx in 0u8..9) {
            let difficulties: Vec<Difficulty> = all::<Difficulty>().collect();
            if let Some(difficulty) = difficulties.get(difficulty_idx as usize) {
                let config = difficulty.create_difficulty_struct();

                // Ghost speed should be in reasonable range
                prop_assert!(config.ghost_speed >= 0.1);
                prop_assert!(config.ghost_speed <= 10.0);
            }
        }
    }

    /// Test difficulty identification methods
    #[test]
    fn test_difficulty_identification() {
        // Test tutorial identification
        assert!(Difficulty::TutorialChapter1.is_tutorial_difficulty());
        assert!(Difficulty::TutorialChapter5.is_tutorial_difficulty());
        assert!(!Difficulty::StandardChallenge.is_tutorial_difficulty());
        assert!(!Difficulty::MasterChallenge.is_tutorial_difficulty());

        // Test enabled status
        for difficulty in all::<Difficulty>() {
            // All difficulties should be enabled by default
            assert!(
                difficulty.is_enabled(),
                "Difficulty {:?} should be enabled",
                difficulty
            );
        }
    }

    /// Test difficulty serialization/deserialization
    #[test]
    fn test_difficulty_serialization() {
        use serde_json;

        for difficulty in all::<Difficulty>() {
            // Test JSON serialization
            let json =
                serde_json::to_string(&difficulty).expect("Should serialize difficulty to JSON");

            let deserialized: Difficulty =
                serde_json::from_str(&json).expect("Should deserialize difficulty from JSON");

            assert_eq!(
                difficulty, deserialized,
                "Difficulty serialization roundtrip failed for {:?}",
                difficulty
            );
        }
    }

    /// Benchmark test for difficulty creation performance
    #[test]
    fn test_difficulty_creation_performance() {
        use std::time::Instant;

        let start = Instant::now();
        for _ in 0..1000 {
            for difficulty in all::<Difficulty>() {
                let _config = difficulty.create_difficulty_struct();
            }
        }
        let duration = start.elapsed();

        // Should complete 1000 iterations of all difficulties in under 100ms
        assert!(
            duration.as_millis() < 100,
            "Difficulty creation too slow: {}ms",
            duration.as_millis()
        );
    }

    /// Test edge cases and boundary conditions
    mod edge_cases {
        use super::*;

        #[test]
        fn test_extreme_difficulty_values() {
            let master = Difficulty::MasterChallenge.create_difficulty_struct();

            // Master difficulty should have reasonable but challenging values
            assert!(
                master.difficulty_score_multiplier >= 1.0,
                "Master difficulty multiplier too low"
            );
            assert!(
                master.difficulty_score_multiplier <= 200.0,
                "Master difficulty multiplier unreasonably high"
            );
        }

        #[test]
        fn test_tutorial_safety() {
            let tutorial = Difficulty::TutorialChapter1.create_difficulty_struct();

            // Tutorial should be forgiving
            assert!(
                tutorial.sanity_drain_rate <= 1.0,
                "Tutorial sanity drain too high"
            );
            assert!(
                tutorial.difficulty_score_multiplier <= 2.0,
                "Tutorial score multiplier too high"
            );
        }
    }

    /// Integration tests with other systems
    mod integration {
        use super::*;

        #[test]
        fn test_difficulty_with_ghost_system() {
            for difficulty in all::<Difficulty>() {
                let config = difficulty.create_difficulty_struct();

                // All ghost sets should produce valid ghost lists
                let ghosts = config.ghost_set.as_vec();
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
