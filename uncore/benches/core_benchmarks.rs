//! Core system benchmarks for Unhaunter game
//!
//! These benchmarks measure performance of critical game systems to ensure
//! they meet performance requirements for production deployment.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use enum_iterator::all;
use uncore::{
    components::board::{boardposition::BoardPosition, direction::Direction, position::Position},
    difficulty::{CurrentDifficulty, Difficulty},
    resources::summary_data::SummaryData,
    types::ghost::types::GhostType,
};

/// Benchmark difficulty system operations
fn benchmark_difficulty_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("difficulty_system");

    // Benchmark difficulty creation
    group.bench_function("create_difficulty_struct", |b| {
        b.iter(|| {
            for difficulty in all::<Difficulty>() {
                black_box(difficulty.create_difficulty_struct());
            }
        })
    });

    // Benchmark difficulty identification
    group.bench_function("is_tutorial_difficulty", |b| {
        b.iter(|| {
            for difficulty in all::<Difficulty>() {
                black_box(difficulty.is_tutorial_difficulty());
            }
        })
    });

    group.finish();
}

/// Benchmark scoring system operations
fn benchmark_scoring_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("scoring_system");

    let difficulty = CurrentDifficulty(Difficulty::StandardChallenge.create_difficulty_struct());

    // Benchmark score calculation with different ghost counts
    for &ghost_count in &[1, 2, 3, 5] {
        group.bench_with_input(
            BenchmarkId::new("calculate_score", ghost_count),
            &ghost_count,
            |b, &ghost_count| {
                b.iter(|| {
                    let ghost_types = vec![GhostType::BeanSidhe; ghost_count];
                    let mut summary = SummaryData::new(ghost_types, difficulty.clone());
                    summary.mission_successful = true;
                    summary.ghosts_unhaunted = ghost_count as u32;
                    summary.player_count = 1;
                    summary.alive_count = 1;
                    summary.average_sanity = 80.0;
                    summary.time_taken_secs = 300.0;
                    summary.repellent_used_amt = 0;
                    black_box(summary.calculate_score());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark position and movement calculations
fn benchmark_position_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_system");

    let pos1 = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        global_z: 0.0,
    };
    let pos2 = Position {
        x: 4.0,
        y: 5.0,
        z: 6.0,
        global_z: 0.0,
    };
    let direction = Direction {
        dx: 3.0,
        dy: 4.0,
        dz: 5.0,
    };

    // Benchmark distance calculations
    group.bench_function("distance", |b| {
        b.iter(|| {
            black_box(pos1.distance(&pos2));
        })
    });

    group.bench_function("distance2", |b| {
        b.iter(|| {
            black_box(pos1.distance2(&pos2));
        })
    });

    group.bench_function("delta", |b| {
        b.iter(|| {
            black_box(pos1.delta(pos2));
        })
    });

    // Benchmark direction operations
    group.bench_function("normalise", |b| {
        b.iter(|| {
            black_box(direction.normalised());
        })
    });

    group.bench_function("add_to_position", |b| {
        b.iter(|| {
            black_box(direction.add_to_position(&pos1));
        })
    });

    group.finish();
}

/// Benchmark board position operations
fn benchmark_board_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_system");

    let board_pos = BoardPosition::from_ndidx((10, 20, 1));

    group.bench_function("to_position", |b| {
        b.iter(|| {
            black_box(board_pos.to_position());
        })
    });

    group.bench_function("to_position_centre", |b| {
        b.iter(|| {
            black_box(board_pos.to_position_centre());
        })
    });

    group.finish();
}

/// Benchmark ghost system operations
fn benchmark_ghost_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("ghost_system");

    // Benchmark ghost evidence access
    group.bench_function("ghost_evidences", |b| {
        b.iter(|| {
            for ghost in all::<GhostType>() {
                black_box(ghost.evidences());
            }
        })
    });

    // Benchmark ghost name access
    group.bench_function("ghost_name", |b| {
        b.iter(|| {
            for ghost in all::<GhostType>() {
                black_box(ghost.name());
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_difficulty_system,
    benchmark_scoring_system,
    benchmark_position_system,
    benchmark_board_system,
    benchmark_ghost_system
);

criterion_main!(benches);
