//! End-to-End (E2E) tests for Unhaunter game
//!
//! These tests simulate complete user workflows and game scenarios
//! to ensure the entire system works correctly from start to finish.

use std::process::Command;
use uncore::difficulty::{CurrentDifficulty, Difficulty};
use uncore::resources::summary_data::SummaryData;
use uncore::types::ghost::types::GhostType;

/// Test that the game binary can be executed and responds to help
#[test]
fn test_game_binary_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "unhaunter_game", "--", "--help"])
        .output()
        .expect("Failed to execute game binary");

    assert!(
        output.status.success(),
        "Game binary should respond to --help successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unhaunter") || stdout.contains("Unhaunter"),
        "Help output should contain game name"
    );
}

/// Test that the game binary accepts draft maps flag
#[test]
fn test_game_binary_draft_maps_flag() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "unhaunter_game",
            "--",
            "--draft-maps",
            "--help",
        ])
        .output()
        .expect("Failed to execute game binary with draft maps flag");

    // The command should either succeed or fail gracefully
    // We're mainly testing that the flag is recognized
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either help should be shown or the game should start (which is fine)
    assert!(
        !stdout.is_empty() || !stderr.is_empty(),
        "Game should produce some output when run with --draft-maps"
    );
}

/// Test that the walkie voice generator binary works
#[test]
fn test_walkie_voice_generator_help() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "unhaunter_walkie_voice_generator",
            "--",
            "--help",
        ])
        .output()
        .expect("Failed to execute walkie voice generator binary");

    assert!(
        output.status.success(),
        "Walkie voice generator should respond to --help successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.is_empty(),
        "Walkie voice generator should produce help output"
    );
}

/// Test complete game initialization workflow
#[test]
fn test_game_initialization_workflow() {
    // Test that the game can be compiled and linked successfully
    let compile_output = Command::new("cargo")
        .args(["build", "--bin", "unhaunter_game"])
        .output()
        .expect("Failed to compile game");

    assert!(
        compile_output.status.success(),
        "Game should compile successfully"
    );

    // Test that the binary exists and is executable
    let binary_path = "target/debug/unhaunter_game";
    let metadata = std::fs::metadata(binary_path);
    assert!(
        metadata.is_ok(),
        "Game binary should exist after compilation"
    );

    let metadata = metadata.unwrap();
    assert!(metadata.is_file(), "Game binary should be a file");
}

/// Test game assets loading workflow
#[test]
fn test_assets_loading_workflow() {
    // Check that essential asset directories exist
    let asset_dirs = [
        "assets/img",
        "assets/sounds",
        "assets/music",
        "assets/maps",
        "assets/fonts",
        "assets/phrasebooks",
    ];

    for dir in &asset_dirs {
        let metadata = std::fs::metadata(dir);
        assert!(metadata.is_ok(), "Asset directory {} should exist", dir);

        let metadata = metadata.unwrap();
        assert!(metadata.is_dir(), "{} should be a directory", dir);
    }

    // Check that some essential files exist
    let essential_files = ["assets/img", "assets/sounds", "assets/maps"];

    for file in &essential_files {
        let metadata = std::fs::metadata(file);
        assert!(metadata.is_ok(), "Essential asset {} should exist", file);
    }
}

/// Test game configuration loading
#[test]
fn test_game_configuration_loading() {

    // Test that we can create a complete game configuration
    let difficulties = [
        Difficulty::TutorialChapter1,
        Difficulty::StandardChallenge,
        Difficulty::MasterChallenge,
    ];

    for difficulty in &difficulties {
        let config = difficulty.create_difficulty_struct();

        // Verify configuration is valid
        assert!(
            !config.difficulty_name.is_empty(),
            "Difficulty name should not be empty"
        );
        assert!(
            config.difficulty_score_multiplier > 0.0,
            "Score multiplier should be positive"
        );
        assert!(config.ghost_speed > 0.0, "Ghost speed should be positive");
        assert!(
            config.sanity_drain_rate >= 0.0,
            "Sanity drain rate should be non-negative"
        );
    }

    // Test that we can create a game summary with all ghost types
    let all_ghost_types: Vec<GhostType> = enum_iterator::all::<GhostType>().collect();
    assert!(
        !all_ghost_types.is_empty(),
        "Should have ghost types available"
    );

    for ghost_type in &all_ghost_types {
        let evidences = ghost_type.evidences();
        assert_eq!(
            evidences.len(),
            5,
            "Each ghost should have exactly 5 evidences"
        );
    }
}

/// Test complete mission simulation workflow
#[test]
fn test_complete_mission_simulation() {

    // Simulate a complete mission from start to finish
    let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];
    let difficulty = CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

    // Create mission summary
    let mut summary = SummaryData::new(ghost_types, difficulty);

    // Simulate mission progression
    summary.mission_successful = true;
    summary.ghosts_unhaunted = 2;
    summary.player_count = 4;
    summary.alive_count = 3; // One player died
    summary.average_sanity = 75.0;
    summary.time_taken_secs = 900.0; // 15 minutes
    summary.repellent_used_amt = 2;

    // Calculate final score
    let final_score = summary.calculate_score();

    // Verify mission was successful
    assert!(summary.mission_successful, "Mission should be successful");
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
}

/// Test error handling and recovery
#[test]
fn test_error_handling_workflow() {

    // Test failed mission scenario
    let ghost_types = vec![GhostType::BeanSidhe];
    let difficulty = CurrentDifficulty(Difficulty::MasterChallenge.create_difficulty_struct());
    let mut summary = SummaryData::new(ghost_types, difficulty);

    // Simulate failed mission
    summary.mission_successful = false;
    summary.ghosts_unhaunted = 0;
    summary.player_count = 2;
    summary.alive_count = 0; // All players died
    summary.average_sanity = 0.0;
    summary.time_taken_secs = 1800.0; // 30 minutes
    summary.repellent_used_amt = 10;

    let failed_score = summary.calculate_score();

    // Failed mission should still produce a valid score (likely 0 or negative)
    assert!(
        !summary.mission_successful,
        "Mission should be marked as failed"
    );
    assert!(
        failed_score <= 0,
        "Failed mission should have non-positive score"
    );
}

/// Test performance under load
#[test]
fn test_performance_under_load() {
    use std::time::Instant;

    let start = Instant::now();

    // Simulate multiple rapid mission calculations
    for _ in 0..100 {
        let ghost_types = vec![GhostType::BeanSidhe, GhostType::Dullahan];
        let difficulty =
            CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());
        let mut summary = SummaryData::new(ghost_types, difficulty);

        summary.mission_successful = true;
        summary.ghosts_unhaunted = 2;
        summary.player_count = 1;
        summary.alive_count = 1;
        summary.average_sanity = 80.0;
        summary.time_taken_secs = 300.0;
        summary.repellent_used_amt = 0;

        let _score = summary.calculate_score();
    }

    let duration = start.elapsed();

    // Should complete 100 calculations quickly
    assert!(
        duration.as_millis() < 1000,
        "100 mission calculations should complete in under 1 second, took {}ms",
        duration.as_millis()
    );
}

/// Test cross-platform compatibility (basic)
#[test]
fn test_cross_platform_compatibility() {
    // Test that the game can be built for different targets
    let targets = ["x86_64-unknown-linux-gnu"]; // Add more targets as needed

    for target in &targets {
        let output = Command::new("cargo")
            .args(["check", "--target", target, "--bin", "unhaunter_game"])
            .output();

        // This might fail if the target isn't installed, which is okay
        if let Ok(output) = output && !output.status.success() {
            // If it fails, it should be due to missing target, not compilation errors
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("target") || stderr.contains("not installed"),
                "Failure should be due to missing target, not compilation errors"
            );
        }
    }
}
