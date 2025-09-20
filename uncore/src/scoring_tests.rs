//! Comprehensive unit tests for the scoring system
//!
//! These tests ensure the scoring calculations are correct, consistent, and handle
//! edge cases properly. They follow NASA-style safety principles with extensive
//! boundary condition testing.

#[cfg(test)]
mod tests {
    use crate::difficulty::{CurrentDifficulty, Difficulty};
    use crate::resources::summary_data::SummaryData;
    use crate::types::ghost::types::GhostType;
    // enum_iterator removed as it's not used in current tests
    use proptest::prelude::*;
    use quickcheck::{TestResult, quickcheck};
    use rstest::*;

    /// Fixture for basic successful mission
    #[fixture]
    fn successful_mission() -> SummaryData {
        let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
        let mut summary = SummaryData::new(ghost_types, difficulty);
        summary.mission_successful = true;
        summary.ghosts_unhaunted = 2;
        summary.player_count = 1;
        summary.alive_count = 1;
        summary.average_sanity = 80.0;
        summary.time_taken_secs = 300.0; // 5 minutes
        summary.repellent_used_amt = 0;
        summary
    }

    /// Fixture for failed mission
    #[fixture]
    fn failed_mission() -> SummaryData {
        let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
        let mut summary = SummaryData::new(ghost_types, difficulty);
        summary.mission_successful = false;
        summary.ghosts_unhaunted = 1;
        summary.player_count = 2;
        summary.alive_count = 1;
        summary.average_sanity = 30.0;
        summary.time_taken_secs = 1800.0; // 30 minutes
        summary.repellent_used_amt = 5;
        summary
    }

    /// Test basic score calculation for successful mission
    #[rstest]
    fn test_successful_mission_scoring(mut successful_mission: SummaryData) {
        let score = successful_mission.calculate_score();

        // Successful mission should have positive score
        assert!(score > 0, "Successful mission should have positive score");

        // Check that score components are calculated
        assert!(
            successful_mission.base_score > 0,
            "Base score should be positive"
        );
        assert!(
            successful_mission.difficulty_multiplier > 0.0,
            "Difficulty multiplier should be positive"
        );
        assert_eq!(
            successful_mission.full_score, score,
            "Full score should match calculated score"
        );
    }

    /// Test score calculation for failed mission
    #[rstest]
    fn test_failed_mission_scoring(mut failed_mission: SummaryData) {
        let score = failed_mission.calculate_score();

        // Failed mission can still have positive score if some ghosts were unhaunted
        assert!(score >= 0, "Score should never be negative");

        // Score should be lower than if all ghosts were unhaunted
        let mut perfect_mission = failed_mission.clone();
        perfect_mission.ghosts_unhaunted = perfect_mission.ghost_types.len() as u32;
        perfect_mission.alive_count = perfect_mission.player_count;
        perfect_mission.average_sanity = 100.0;
        perfect_mission.repellent_used_amt = 0;
        perfect_mission.time_taken_secs = 60.0;

        let perfect_score = perfect_mission.calculate_score();
        assert!(
            score <= perfect_score,
            "Failed mission score ({}) should not exceed perfect score ({})",
            score,
            perfect_score
        );
    }

    /// Test score scaling with difficulty
    #[test]
    fn test_difficulty_score_scaling() {
        let ghost_types = vec![GhostType::BeanSidhe];
        let difficulties = [
            Difficulty::TutorialChapter1,
            Difficulty::StandardChallenge,
            Difficulty::MasterChallenge,
        ];

        let mut scores = Vec::new();

        for difficulty in difficulties.iter() {
            let current_difficulty = CurrentDifficulty(difficulty.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types.clone(), current_difficulty);
            summary.mission_successful = true;
            summary.ghosts_unhaunted = 1;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = 0;

            let score = summary.calculate_score();
            scores.push((difficulty, score));
        }

        // Higher difficulty should generally yield higher scores
        assert!(
            scores[2].1 > scores[0].1,
            "Master difficulty ({}) should score higher than Tutorial ({})",
            scores[2].1,
            scores[0].1
        );
    }

    /// Test sanity impact on scoring
    #[rstest]
    #[case(100, "perfect sanity")]
    #[case(80, "good sanity")]
    #[case(50, "average sanity")]
    #[case(20, "low sanity")]
    #[case(0, "zero sanity")]
    fn test_sanity_impact_on_score(#[case] sanity: u32, #[case] _description: &str) {
        let ghost_types = vec![GhostType::BeanSidhe];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
        let mut summary = SummaryData::new(ghost_types, difficulty);

        summary.mission_successful = true;
        summary.ghosts_unhaunted = 1;
        summary.player_count = 1;
        summary.alive_count = 1;
        summary.average_sanity = sanity as f32;
        summary.time_taken_secs = 300.0;
        summary.repellent_used_amt = 0;

        let score = summary.calculate_score();

        // Score should always be non-negative
        assert!(
            score >= 0,
            "Score should never be negative for sanity {}",
            sanity
        );

        // Higher sanity should generally yield higher scores (within reasonable bounds)
        if sanity > 0 {
            assert!(score > 0, "Positive sanity should yield positive score");
        }
    }

    /// Test repellent usage penalty
    #[test]
    fn test_repellent_usage_penalty() {
        let ghost_types = vec![GhostType::BeanSidhe];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

        let base_summary = |repellent_used: u32| {
            let mut summary = SummaryData::new(ghost_types.clone(), difficulty.clone());
            summary.mission_successful = true;
            summary.ghosts_unhaunted = 1;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = repellent_used;
            summary
        };

        let no_repellent_score = base_summary(0).calculate_score();
        let with_repellent_score = base_summary(5).calculate_score();

        assert!(
            no_repellent_score >= with_repellent_score,
            "Using repellent should not increase score: {} vs {}",
            no_repellent_score,
            with_repellent_score
        );
    }

    /// Test time bonus calculation
    #[test]
    fn test_time_bonus() {
        let ghost_types = vec![GhostType::BeanSidhe];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

        let base_summary = |time_secs: u32| {
            let mut summary = SummaryData::new(ghost_types.clone(), difficulty.clone());
            summary.mission_successful = true;
            summary.ghosts_unhaunted = 1;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = time_secs as f32;
            summary.repellent_used_amt = 0;
            summary
        };

        let fast_score = base_summary(60).calculate_score(); // 1 minute
        let slow_score = base_summary(1800).calculate_score(); // 30 minutes

        assert!(
            fast_score >= slow_score,
            "Faster completion should not decrease score: {} vs {}",
            fast_score,
            slow_score
        );
    }

    /// Test survival impact on scoring
    #[test]
    fn test_survival_impact() {
        let ghost_types = vec![GhostType::BeanSidhe];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

        let base_summary = |player_count: u32, alive_count: u32| {
            let mut summary = SummaryData::new(ghost_types.clone(), difficulty.clone());
            summary.mission_successful = true;
            summary.ghosts_unhaunted = 1;
            summary.player_count = player_count as usize;
            summary.alive_count = alive_count as usize;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = 0;
            summary
        };

        let all_survive_score = base_summary(2, 2).calculate_score();
        let one_dies_score = base_summary(2, 1).calculate_score();

        assert!(
            all_survive_score >= one_dies_score,
            "All players surviving should not decrease score: {} vs {}",
            all_survive_score,
            one_dies_score
        );
    }

    quickcheck! {
        fn prop_score_never_negative(
            ghosts_unhaunted: u32,
            player_count: u32,
            alive_count: u32,
            sanity: u32,
            time_secs: u32,
            repellent_used: u32
        ) -> TestResult {
            // Ensure reasonable bounds
            if ghosts_unhaunted > 10 || player_count == 0 || player_count > 10 ||
               alive_count > player_count || sanity > 100 ||
               time_secs == 0 || time_secs > 7200 || repellent_used > 50 {
                return TestResult::discard();
            }

            let ghost_types = vec![GhostType::BeanSidhe; ghosts_unhaunted.min(5) as usize];
            if ghost_types.is_empty() {
                return TestResult::discard();
            }

            let difficulty = CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types, difficulty);
            summary.mission_successful = true;
            summary.ghosts_unhaunted = ghosts_unhaunted.min(summary.ghost_types.len() as u32);
            summary.player_count = player_count as usize;
            summary.alive_count = alive_count as usize;
            summary.average_sanity = sanity as f32;
            summary.time_taken_secs = time_secs as f32;
            summary.repellent_used_amt = repellent_used;

            let score = summary.calculate_score();

            TestResult::from_bool(score >= 0)
        }
    }

    proptest! {
        #[test]
        fn prop_score_within_reasonable_bounds(
            ghosts_unhaunted in 0u32..=5,
            sanity in 0u32..=100,
            time_secs in 60u32..=3600,
        ) {
            let ghost_types = vec![GhostType::BeanSidhe; 1.max(ghosts_unhaunted as usize)];
            let difficulty = CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types, difficulty);

            summary.mission_successful = true;
            summary.ghosts_unhaunted = ghosts_unhaunted.min(summary.ghost_types.len() as u32);
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = sanity as f32;
            summary.time_taken_secs = time_secs as f32;
            summary.repellent_used_amt = 0;

            let score = summary.calculate_score();

            // Score should be within reasonable bounds
            prop_assert!(score >= 0);
            prop_assert!(score <= 1_000_000); // Upper bound from clamp in calculate_score
        }
    }

    /// Test edge cases and boundary conditions
    mod edge_cases {
        use super::*;

        #[test]
        fn test_zero_ghosts_unhaunted() {
            let ghost_types = vec![GhostType::BeanSidhe];
            let difficulty =
                CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types, difficulty);

            summary.ghosts_unhaunted = 0;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = 0;

            let score = summary.calculate_score();

            // Should handle zero ghosts unhaunted gracefully
            assert!(
                score >= 0,
                "Score should be non-negative even with zero ghosts unhaunted"
            );
        }

        #[test]
        fn test_all_players_dead() {
            let ghost_types = vec![GhostType::BeanSidhe];
            let difficulty =
                CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types, difficulty);

            summary.ghosts_unhaunted = 1;
            summary.player_count = 2;
            summary.alive_count = 0;
            summary.average_sanity = 0.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = 0;

            let score = summary.calculate_score();

            // Should handle all players dead gracefully
            assert!(
                score >= 0,
                "Score should be non-negative even with all players dead"
            );
        }

        #[test]
        fn test_extreme_repellent_usage() {
            let ghost_types = vec![GhostType::BeanSidhe];
            let difficulty =
                CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types, difficulty);

            summary.ghosts_unhaunted = 1;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 300.0;
            summary.repellent_used_amt = 1000; // Extreme usage

            let score = summary.calculate_score();

            // Should handle extreme repellent usage without panic
            assert!(
                score >= 0,
                "Score should be non-negative even with extreme repellent usage"
            );
        }

        #[test]
        fn test_very_long_mission() {
            let ghost_types = vec![GhostType::BeanSidhe];
            let difficulty =
                CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
            let mut summary = SummaryData::new(ghost_types, difficulty);

            summary.ghosts_unhaunted = 1;
            summary.player_count = 1;
            summary.alive_count = 1;
            summary.average_sanity = 80.0;
            summary.time_taken_secs = 86400.0; // 24 hours
            summary.repellent_used_amt = 0;

            let score = summary.calculate_score();

            // Should handle very long missions gracefully
            assert!(
                score >= 0,
                "Score should be non-negative for very long missions"
            );
        }
    }

    /// Performance and stress tests
    mod performance {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_scoring_performance() {
            let ghost_types = vec![GhostType::BeanSidhe];
            let difficulty =
                CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

            let start = Instant::now();
            for _ in 0..10000 {
                let mut summary = SummaryData::new(ghost_types.clone(), difficulty.clone());
                summary.ghosts_unhaunted = 1;
                summary.player_count = 1;
                summary.alive_count = 1;
                summary.average_sanity = 80.0;
                summary.time_taken_secs = 300.0;
                summary.repellent_used_amt = 0;
                let _score = summary.calculate_score();
            }
            let duration = start.elapsed();

            // Should complete 10,000 score calculations in under 100ms
            assert!(
                duration.as_millis() < 100,
                "Score calculation too slow: {}ms for 10,000 calculations",
                duration.as_millis()
            );
        }
    }
}
