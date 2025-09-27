//! ## Performance Optimizer Module
//!
//! This module provides performance optimization utilities and monitoring
//! to improve game performance and identify bottlenecks.

use bevy::prelude::*;
use std::collections::HashMap;

/// Resource for tracking performance metrics
#[derive(Resource, Default)]
pub struct PerformanceMonitor {
    /// Frame time measurements
    pub frame_times: Vec<f32>,
    /// System execution times
    pub system_times: HashMap<String, Vec<f32>>,
    /// Maximum number of samples to keep
    pub max_samples: usize,
    /// Performance warnings
    pub warnings: Vec<PerformanceWarning>,
}

/// Performance warning types
#[derive(Debug, Clone)]
pub enum PerformanceWarning {
    HighFrameTime(f32),
    SlowSystem(String, f32),
    MemoryPressure(usize),
}

impl PerformanceMonitor {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_samples),
            system_times: HashMap::new(),
            max_samples,
            warnings: Vec::new(),
        }
    }
    
    /// Record a frame time measurement
    pub fn record_frame_time(&mut self, frame_time: f32) {
        self.frame_times.push(frame_time);
        
        // Keep only the most recent samples
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
        
        // Check for performance warnings
        if frame_time > 0.033 { // > 30fps threshold
            self.warnings.push(PerformanceWarning::HighFrameTime(frame_time));
        }
    }
    
    /// Record system execution time
    pub fn record_system_time(&mut self, system_name: String, execution_time: f32) {
        let times = self.system_times.entry(system_name.clone()).or_insert_with(Vec::new);
        times.push(execution_time);
        
        // Keep only recent samples
        if times.len() > self.max_samples {
            times.remove(0);
        }
        
        // Check for slow systems
        if execution_time > 0.016 { // > 1ms threshold
            self.warnings.push(PerformanceWarning::SlowSystem(system_name, execution_time));
        }
    }
    
    /// Get average frame time
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.frame_times.iter().sum();
        sum / self.frame_times.len() as f32
    }
    
    /// Get frame rate
    pub fn frame_rate(&self) -> f32 {
        let avg_frame_time = self.average_frame_time();
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }
    
    /// Clear warnings older than specified time
    pub fn clear_old_warnings(&mut self, max_warnings: usize) {
        if self.warnings.len() > max_warnings {
            self.warnings.drain(0..self.warnings.len() - max_warnings);
        }
    }
    
    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            frame_rate: self.frame_rate(),
            average_frame_time: self.average_frame_time(),
            warning_count: self.warnings.len(),
            system_count: self.system_times.len(),
        }
    }
}

/// Performance summary data
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub frame_rate: f32,
    pub average_frame_time: f32,
    pub warning_count: usize,
    pub system_count: usize,
}

/// System that monitors frame performance
pub fn monitor_frame_performance(
    mut monitor: ResMut<PerformanceMonitor>,
    time: Res<Time>,
) {
    let frame_time = time.delta_secs();
    monitor.record_frame_time(frame_time);
    monitor.clear_old_warnings(50); // Keep only 50 most recent warnings
}

/// System that reports performance statistics periodically
pub fn report_performance_stats(
    monitor: Res<PerformanceMonitor>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Report every 10 seconds
    if *timer >= 10.0 {
        *timer = 0.0;
        
        let summary = monitor.get_summary();
        info!(
            "Performance: {:.1} FPS, {:.3}ms avg frame time, {} warnings, {} systems tracked",
            summary.frame_rate,
            summary.average_frame_time * 1000.0,
            summary.warning_count,
            summary.system_count
        );
        
        // Report recent warnings
        for warning in monitor.warnings.iter().rev().take(5) {
            match warning {
                PerformanceWarning::HighFrameTime(frame_time) => {
                    warn!("High frame time: {:.3}ms", frame_time * 1000.0);
                }
                PerformanceWarning::SlowSystem(name, time) => {
                    warn!("Slow system '{}': {:.3}ms", name, time * 1000.0);
                }
                PerformanceWarning::MemoryPressure(pressure) => {
                    warn!("Memory pressure: {} bytes", pressure);
                }
            }
        }
    }
}

/// Component for marking entities that should be optimized
#[derive(Component, Debug, Clone)]
pub struct PerformanceOptimized {
    /// Whether this entity should be culled when off-screen
    pub cullable: bool,
    /// Whether this entity should use LOD (Level of Detail)
    pub use_lod: bool,
    /// Whether this entity should be pooled
    pub poolable: bool,
}

impl Default for PerformanceOptimized {
    fn default() -> Self {
        Self {
            cullable: true,
            use_lod: false,
            poolable: false,
        }
    }
}

/// Simple frustum culling system for performance optimization
pub fn frustum_culling(
    mut query: Query<(&mut Visibility, &Transform, &PerformanceOptimized)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    // Get the main camera (simplified - just use the first camera)
    let Some((_camera, camera_transform)) = cameras.iter().next() else {
        return;
    };
    
    // Simple distance-based culling (can be enhanced with proper frustum culling)
    let cull_distance = 100.0; // 100 units
    
    for (mut visibility, transform, optimized) in query.iter_mut() {
        if !optimized.cullable {
            continue;
        }
        
        let distance = camera_transform.translation().distance(transform.translation);
        
        if distance > cull_distance {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

/// System that monitors memory usage
pub fn monitor_memory_usage(
    mut monitor: ResMut<PerformanceMonitor>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Check memory every 30 seconds
    if *timer >= 30.0 {
        *timer = 0.0;
        
        // Simple memory pressure detection
        // In a real implementation, you'd use system-specific APIs
        let memory_pressure = std::process::id() as usize; // Placeholder
        
        if memory_pressure > 1_000_000_000 { // 1GB threshold
            monitor.warnings.push(PerformanceWarning::MemoryPressure(memory_pressure));
        }
    }
}

/// Helper function to create a performance-optimized entity
pub fn spawn_optimized_entity(
    commands: &mut Commands,
    position: Vec3,
    cullable: bool,
    poolable: bool,
) -> Entity {
    commands.spawn((
        Transform::from_translation(position),
        PerformanceOptimized {
            cullable,
            poolable,
            use_lod: false,
        },
        Visibility::Visible,
    )).id()
}

/// Macro for timing system execution
#[macro_export]
macro_rules! time_system {
    ($name:expr, $system:expr) => {{
        let start = std::time::Instant::now();
        let result = $system;
        let duration = start.elapsed();
        
        // Log if system takes too long
        if duration.as_secs_f32() > 0.001 { // 1ms threshold
            warn!("Slow system '{}': {:.3}ms", $name, duration.as_secs_f32() * 1000.0);
        }
        
        result
    }};
}
