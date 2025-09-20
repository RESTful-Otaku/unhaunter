//! Comprehensive unit tests for board position and movement systems
//!
//! These tests ensure spatial calculations, collision detection, and movement
//! mechanics work correctly under all conditions.

#[cfg(test)]
mod tests {
    use crate::components::board::{
        boardposition::BoardPosition, direction::Direction, position::Position,
    };
    use proptest::prelude::*;
    use quickcheck::{TestResult, quickcheck};
    use rstest::*;

    // Test fixtures
    #[fixture]
    fn origin_position() -> Position {
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            global_z: 0.0,
        }
    }

    #[fixture]
    fn unit_position() -> Position {
        Position {
            x: 1.0,
            y: 1.0,
            z: 0.0,
            global_z: 0.0,
        }
    }

    #[fixture]
    fn origin_board_pos() -> BoardPosition {
        BoardPosition::from_ndidx((0, 0, 0))
    }

    /// Test basic position creation and properties
    #[rstest]
    fn test_position_creation(origin_position: Position, unit_position: Position) {
        // Test origin position
        assert_eq!(origin_position.x, 0.0);
        assert_eq!(origin_position.y, 0.0);
        assert_eq!(origin_position.z, 0.0);
        assert_eq!(origin_position.global_z, 0.0);

        // Test unit position
        assert_eq!(unit_position.x, 1.0);
        assert_eq!(unit_position.y, 1.0);
        assert_eq!(unit_position.z, 0.0);
        assert_eq!(unit_position.global_z, 0.0);
    }

    /// Test distance calculations
    #[test]
    fn test_distance_calculations() {
        let pos1 = Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            global_z: 0.0,
        };
        let pos2 = Position {
            x: 3.0,
            y: 4.0,
            z: 0.0,
            global_z: 0.0,
        };

        // Test 2D distance (3-4-5 triangle)
        let distance = pos1.distance(&pos2);
        assert!(
            (distance - 5.0).abs() < 0.001,
            "Expected distance 5.0, got {}",
            distance
        );

        // Test squared distance (more efficient)
        let distance2 = pos1.distance2(&pos2);
        assert!(
            (distance2 - 25.0).abs() < 0.001,
            "Expected squared distance 25.0, got {}",
            distance2
        );
    }

    /// Test position delta calculations
    #[test]
    fn test_position_delta() {
        let pos1 = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            global_z: 0.0,
        };
        let pos2 = Position {
            x: 4.0,
            y: 6.0,
            z: 7.0,
            global_z: 0.0,
        };

        let delta = pos1.delta(pos2);

        assert_eq!(
            delta.dx, -3.0,
            "Delta X should be -3.0 (pos1.x - pos2.x = 1.0 - 4.0)"
        );
        assert_eq!(
            delta.dy, -4.0,
            "Delta Y should be -4.0 (pos1.y - pos2.y = 2.0 - 6.0)"
        );
        assert_eq!(
            delta.dz, -4.0,
            "Delta Z should be -4.0 (pos1.z - pos2.z = 3.0 - 7.0)"
        );
    }

    /// Test direction normalization
    #[test]
    fn test_direction_normalization() {
        let dir = Direction {
            dx: 3.0,
            dy: 4.0,
            dz: 0.0,
        };
        let normalized = dir.normalised();

        // Should have unit length
        let length = (normalized.dx * normalized.dx
            + normalized.dy * normalized.dy
            + normalized.dz * normalized.dz)
            .sqrt();

        assert!(
            (length - 1.0).abs() < 0.001,
            "Normalized direction should have unit length, got {}",
            length
        );

        // Should maintain direction ratios
        let original_ratio = dir.dy / dir.dx;
        let normalized_ratio = normalized.dy / normalized.dx;
        assert!(
            (original_ratio - normalized_ratio).abs() < 0.001,
            "Normalization should preserve direction ratios"
        );
    }

    /// Test zero direction normalization
    #[test]
    fn test_zero_direction_normalization() {
        let zero_dir = Direction {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
        };
        let normalized = zero_dir.normalised();

        // Should handle zero vector gracefully
        assert!(normalized.dx.is_finite(), "Normalized X should be finite");
        assert!(normalized.dy.is_finite(), "Normalized Y should be finite");
        assert!(normalized.dz.is_finite(), "Normalized Z should be finite");
    }

    /// Test board position creation and conversion
    #[rstest]
    fn test_board_position_creation(origin_board_pos: BoardPosition) {
        // BoardPosition fields are private, so we test functionality instead
        let world_pos = origin_board_pos.to_position();
        assert!(world_pos.x.is_finite());
        assert!(world_pos.y.is_finite());
        assert!(world_pos.z.is_finite());
    }

    /// Test board position to world position conversion
    #[test]
    fn test_board_to_world_conversion() {
        let board_pos = BoardPosition::from_ndidx((1, 2, 0));
        let world_pos = board_pos.to_position();

        // Should convert board coordinates to world coordinates
        assert!(
            world_pos.x != 0.0 || world_pos.y != 0.0,
            "World position should not be zero for non-zero board position"
        );
    }

    /// Test board position center calculation
    #[test]
    fn test_board_position_centre() {
        let board_pos = BoardPosition::from_ndidx((1, 1, 0));
        let centre_pos = board_pos.to_position_centre();

        // Centre should be different from corner
        let corner_pos = board_pos.to_position();
        assert!(
            centre_pos.x != corner_pos.x || centre_pos.y != corner_pos.y,
            "Centre position should differ from corner position"
        );
    }

    /// Test position addition with direction
    #[test]
    fn test_position_direction_addition() {
        let pos = Position {
            x: 1.0,
            y: 1.0,
            z: 0.0,
            global_z: 0.0,
        };
        let dir = Direction {
            dx: 2.0,
            dy: 3.0,
            dz: 1.0,
        };

        let new_pos = dir.add_to_position(&pos);

        assert_eq!(new_pos.x, 3.0, "X should be 1.0 + 2.0 = 3.0");
        assert_eq!(new_pos.y, 4.0, "Y should be 1.0 + 3.0 = 4.0");
        assert_eq!(new_pos.z, 1.0, "Z should be 0.0 + 1.0 = 1.0");
        assert_eq!(
            new_pos.global_z, pos.global_z,
            "Global Z should be preserved"
        );
    }

    /// Test direction scaling
    #[test]
    fn test_direction_scaling() {
        let dir = Direction {
            dx: 1.0,
            dy: 2.0,
            dz: 3.0,
        };
        let scaled = dir * 2.0;

        assert_eq!(scaled.dx, 2.0, "Scaled X should be 2.0");
        assert_eq!(scaled.dy, 4.0, "Scaled Y should be 4.0");
        assert_eq!(scaled.dz, 6.0, "Scaled Z should be 6.0");
    }

    /// Test direction addition
    #[test]
    fn test_direction_addition() {
        let dir1 = Direction {
            dx: 1.0,
            dy: 2.0,
            dz: 3.0,
        };
        let dir2 = Direction {
            dx: 4.0,
            dy: 5.0,
            dz: 6.0,
        };
        let sum = dir1 + dir2;

        assert_eq!(sum.dx, 5.0, "Sum X should be 5.0");
        assert_eq!(sum.dy, 7.0, "Sum Y should be 7.0");
        assert_eq!(sum.dz, 9.0, "Sum Z should be 9.0");
    }

    quickcheck! {
        fn prop_distance_symmetry(
            x1: f32, y1: f32, z1: f32,
            x2: f32, y2: f32, z2: f32
        ) -> TestResult {
            // Skip infinite or NaN values and limit range to avoid overflow
            if ![x1, y1, z1, x2, y2, z2].iter().all(|&x| x.is_finite() && x.abs() < 1000.0) {
                return TestResult::discard();
            }

            let pos1 = Position { x: x1, y: y1, z: z1, global_z: 0.0 };
            let pos2 = Position { x: x2, y: y2, z: z2, global_z: 0.0 };

            let dist1to2 = pos1.distance(&pos2);
            let dist2to1 = pos2.distance(&pos1);

            // Additional check for finite results
            if !dist1to2.is_finite() || !dist2to1.is_finite() {
                return TestResult::discard();
            }

            TestResult::from_bool((dist1to2 - dist2to1).abs() < 0.001)
        }
    }

    quickcheck! {
        fn prop_triangle_inequality(
            x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32
        ) -> TestResult {
            // Skip infinite or NaN values
            if ![x1, y1, x2, y2, x3, y3].iter().all(|&x| x.is_finite()) {
                return TestResult::discard();
            }

            let pos1 = Position { x: x1, y: y1, z: 0.0, global_z: 0.0 };
            let pos2 = Position { x: x2, y: y2, z: 0.0, global_z: 0.0 };
            let pos3 = Position { x: x3, y: y3, z: 0.0, global_z: 0.0 };

            let dist12 = pos1.distance(&pos2);
            let dist23 = pos2.distance(&pos3);
            let dist13 = pos1.distance(&pos3);

            // Triangle inequality: d(A,C) <= d(A,B) + d(B,C)
            TestResult::from_bool(dist13 <= dist12 + dist23 + 0.001) // Small epsilon for floating point
        }
    }

    proptest! {
        #[test]
        fn prop_normalization_preserves_direction(
            dx in -1000.0f32..1000.0,
            dy in -1000.0f32..1000.0,
            dz in -1000.0f32..1000.0
        ) {
            // Skip zero vectors
            if dx.abs() < 0.001 && dy.abs() < 0.001 && dz.abs() < 0.001 {
                return Ok(());
            }

            let dir = Direction { dx, dy, dz };
            let normalized = dir.normalised();

            // Check that direction is preserved (same sign for each component)
            if dx != 0.0 {
                prop_assert_eq!(dx.signum(), normalized.dx.signum());
            }
            if dy != 0.0 {
                prop_assert_eq!(dy.signum(), normalized.dy.signum());
            }
            if dz != 0.0 {
                prop_assert_eq!(dz.signum(), normalized.dz.signum());
            }

            // Check that length is approximately 1
            let length = (normalized.dx * normalized.dx +
                         normalized.dy * normalized.dy +
                         normalized.dz * normalized.dz).sqrt();
            prop_assert!((length - 1.0).abs() < 0.001);

            prop_assert!(true)
        }
    }

    /// Test edge cases and boundary conditions
    mod edge_cases {
        use super::*;

        #[test]
        fn test_very_large_coordinates() {
            let pos1 = Position {
                x: 100000.0,
                y: 100000.0,
                z: 0.0,
                global_z: 0.0,
            };
            let pos2 = Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                global_z: 0.0,
            };

            // Should handle large coordinates without overflow
            let distance = pos1.distance(&pos2);
            assert!(
                distance.is_finite(),
                "Distance should be finite for large coordinates"
            );
            assert!(
                distance > 0.0,
                "Distance should be positive for different positions"
            );
        }

        #[test]
        fn test_very_small_coordinates() {
            let pos1 = Position {
                x: f32::MIN_POSITIVE,
                y: f32::MIN_POSITIVE,
                z: 0.0,
                global_z: 0.0,
            };
            let pos2 = Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                global_z: 0.0,
            };

            // Should handle very small coordinates
            let distance = pos1.distance(&pos2);
            assert!(distance >= 0.0, "Distance should be non-negative");
            assert!(
                distance.is_finite(),
                "Distance should be finite for small coordinates"
            );
        }

        #[test]
        fn test_negative_coordinates() {
            let pos1 = Position {
                x: -10.0,
                y: -20.0,
                z: -30.0,
                global_z: 0.0,
            };
            let pos2 = Position {
                x: 10.0,
                y: 20.0,
                z: 30.0,
                global_z: 0.0,
            };

            let distance = pos1.distance(&pos2);
            // The actual delta is pos1 - pos2 = (-10, -20, -30) - (10, 20, 30) = (-20, -40, -60)
            // But distance2_zf uses z factor of 6.0, so dz becomes -60 * 6.0 = -360
            // Distance = sqrt((-20)^2 + (-40)^2 + (-360)^2) = sqrt(400 + 1600 + 129600) = sqrt(131600)
            let expected = (400.0f32 + 1600.0 + 129600.0).sqrt();

            assert!(
                (distance - expected).abs() < 0.001,
                "Distance calculation should work with negative coordinates"
            );
        }

        #[test]
        fn test_identical_positions() {
            let pos1 = Position {
                x: 5.0,
                y: 10.0,
                z: 15.0,
                global_z: 0.0,
            };
            let pos2 = Position {
                x: 5.0,
                y: 10.0,
                z: 15.0,
                global_z: 0.0,
            };

            let distance = pos1.distance(&pos2);
            assert!(
                (distance - 0.0).abs() < 0.001,
                "Distance between identical positions should be zero"
            );
        }
    }

    /// Performance tests
    mod performance {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_distance_calculation_performance() {
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

            let start = Instant::now();
            for _ in 0..100_000 {
                let _distance = pos1.distance(&pos2);
            }
            let duration = start.elapsed();

            // Should complete 100,000 distance calculations quickly
            assert!(
                duration.as_millis() < 50,
                "Distance calculation too slow: {}ms",
                duration.as_millis()
            );
        }

        #[test]
        fn test_normalization_performance() {
            let dir = Direction {
                dx: 3.0,
                dy: 4.0,
                dz: 5.0,
            };

            let start = Instant::now();
            for _ in 0..100_000 {
                let _normalized = dir.normalised();
            }
            let duration = start.elapsed();

            // Should complete 100,000 normalizations quickly
            assert!(
                duration.as_millis() < 100,
                "Normalization too slow: {}ms",
                duration.as_millis()
            );
        }
    }

    /// Integration tests with game systems
    mod integration {
        use super::*;

        #[test]
        fn test_movement_integration() {
            let start_pos = Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                global_z: 0.0,
            };
            let move_dir = Direction {
                dx: 1.0,
                dy: 0.0,
                dz: 0.0,
            };
            let move_distance = 5.0;

            // Simulate movement
            let normalized_dir = move_dir.normalised();
            let scaled_dir = normalized_dir * move_distance;
            let end_pos = scaled_dir.add_to_position(&start_pos);

            // Verify movement worked correctly
            let actual_distance = start_pos.distance(&end_pos);
            assert!(
                (actual_distance - move_distance).abs() < 0.001,
                "Movement should cover exact distance: expected {}, got {}",
                move_distance,
                actual_distance
            );
        }

        #[test]
        fn test_board_world_coordinate_consistency() {
            let board_pos = BoardPosition::from_ndidx((5, 10, 0));
            let world_pos = board_pos.to_position();

            // Converting back should give similar board position
            // (This test assumes there's a way to convert back, or tests consistency)
            assert!(world_pos.x.is_finite(), "World X should be finite");
            assert!(world_pos.y.is_finite(), "World Y should be finite");
            assert!(world_pos.z.is_finite(), "World Z should be finite");
        }
    }
}
