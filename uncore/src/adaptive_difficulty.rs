//! ## Adaptive Difficulty System
//!
//! This module provides an adaptive difficulty system that dynamically adjusts
//! game difficulty based on player performance and behavior patterns.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::difficulty::Difficulty;

/// Resource for tracking player performance and adjusting difficulty
#[derive(Resource, Debug, Clone)]
pub struct AdaptiveDifficultyManager {
    /// Current base difficulty
    pub base_difficulty: Difficulty,
    /// Current adaptive difficulty multiplier (1.0 = base, >1.0 = harder, <1.0 = easier)
    pub adaptive_multiplier: f32,
    /// Player performance metrics
    pub performance_metrics: PlayerPerformanceMetrics,
    /// Difficulty adjustment history
    pub adjustment_history: VecDeque<DifficultyAdjustment>,
    /// Whether adaptive difficulty is enabled
    pub enabled: bool,
    /// Minimum adaptive multiplier
    pub min_multiplier: f32,
    /// Maximum adaptive multiplier
    pub max_multiplier: f32,
    /// How quickly to adjust difficulty (0.0 = instant, 1.0 = very slow)
    pub adjustment_speed: f32,
}

/// Player performance metrics for adaptive difficulty
#[derive(Debug, Clone, Default)]
pub struct PlayerPerformanceMetrics {
    /// Recent mission completion times (in seconds)
    pub completion_times: VecDeque<f32>,
    /// Recent death counts per mission
    pub death_counts: VecDeque<u32>,
    /// Recent evidence collection accuracy (0.0 to 1.0)
    pub evidence_accuracy: VecDeque<f32>,
    /// Recent ghost identification accuracy (0.0 to 1.0)
    pub ghost_identification_accuracy: VecDeque<f32>,
    /// Recent equipment usage efficiency (0.0 to 1.0)
    pub equipment_efficiency: VecDeque<f32>,
    /// Recent time spent in missions (in seconds)
    pub mission_durations: VecDeque<f32>,
    /// Recent success/failure ratio
    pub success_rate: VecDeque<bool>,
    /// Maximum number of samples to keep
    pub max_samples: usize,
}

/// Record of a difficulty adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAdjustment {
    pub timestamp: f64,
    pub old_multiplier: f32,
    pub new_multiplier: f32,
    pub reason: AdjustmentReason,
    pub performance_data: String,
}

/// Reasons for difficulty adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentReason {
    PlayerStruggling,
    PlayerDominating,
    MissionTooLong,
    MissionTooShort,
    TooManyDeaths,
    TooFewDeaths,
    PoorEvidenceCollection,
    ExcellentEvidenceCollection,
    PoorGhostIdentification,
    ExcellentGhostIdentification,
    InefficientEquipmentUse,
    EfficientEquipmentUse,
}

impl Default for AdaptiveDifficultyManager {
    fn default() -> Self {
        Self {
            base_difficulty: Difficulty::StandardChallenge,
            adaptive_multiplier: 1.0,
            performance_metrics: PlayerPerformanceMetrics::new(20), // Keep 20 recent samples
            adjustment_history: VecDeque::new(),
            enabled: true,
            min_multiplier: 0.5,  // 50% easier than base
            max_multiplier: 2.0,  // 200% harder than base
            adjustment_speed: 0.1, // Gradual adjustments
        }
    }
}

impl PlayerPerformanceMetrics {
    pub fn new(max_samples: usize) -> Self {
        Self {
            completion_times: VecDeque::with_capacity(max_samples),
            death_counts: VecDeque::with_capacity(max_samples),
            evidence_accuracy: VecDeque::with_capacity(max_samples),
            ghost_identification_accuracy: VecDeque::with_capacity(max_samples),
            equipment_efficiency: VecDeque::with_capacity(max_samples),
            mission_durations: VecDeque::with_capacity(max_samples),
            success_rate: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }
    
    /// Record mission completion time
    pub fn record_completion_time(&mut self, time: f32) {
        if self.completion_times.len() >= self.max_samples {
            self.completion_times.pop_front();
        }
        self.completion_times.push_back(time);
    }
    
    /// Record death count for a mission
    pub fn record_deaths(&mut self, deaths: u32) {
        if self.death_counts.len() >= self.max_samples {
            self.death_counts.pop_front();
        }
        self.death_counts.push_back(deaths);
    }
    
    /// Record evidence collection accuracy
    pub fn record_evidence_accuracy(&mut self, accuracy: f32) {
        if self.evidence_accuracy.len() >= self.max_samples {
            self.evidence_accuracy.pop_front();
        }
        self.evidence_accuracy.push_back(accuracy);
    }
    
    /// Record ghost identification accuracy
    pub fn record_ghost_identification_accuracy(&mut self, accuracy: f32) {
        if self.ghost_identification_accuracy.len() >= self.max_samples {
            self.ghost_identification_accuracy.pop_front();
        }
        self.ghost_identification_accuracy.push_back(accuracy);
    }
    
    /// Record equipment usage efficiency
    pub fn record_equipment_efficiency(&mut self, efficiency: f32) {
        if self.equipment_efficiency.len() >= self.max_samples {
            self.equipment_efficiency.pop_front();
        }
        self.equipment_efficiency.push_back(efficiency);
    }
    
    /// Record mission duration
    pub fn record_mission_duration(&mut self, duration: f32) {
        if self.mission_durations.len() >= self.max_samples {
            self.mission_durations.pop_front();
        }
        self.mission_durations.push_back(duration);
    }
    
    /// Record mission success/failure
    pub fn record_mission_result(&mut self, success: bool) {
        if self.success_rate.len() >= self.max_samples {
            self.success_rate.pop_front();
        }
        self.success_rate.push_back(success);
    }
    
    
    /// Get average completion time
    pub fn average_completion_time(&self) -> f32 {
        if self.completion_times.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.completion_times.iter().sum();
        sum / self.completion_times.len() as f32
    }
    
    /// Get average death count
    pub fn average_deaths(&self) -> f32 {
        if self.death_counts.is_empty() {
            return 0.0;
        }
        let sum: u32 = self.death_counts.iter().sum();
        sum as f32 / self.death_counts.len() as f32
    }
    
    /// Get average evidence accuracy
    pub fn average_evidence_accuracy(&self) -> f32 {
        if self.evidence_accuracy.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.evidence_accuracy.iter().sum();
        sum / self.evidence_accuracy.len() as f32
    }
    
    /// Get average ghost identification accuracy
    pub fn average_ghost_identification_accuracy(&self) -> f32 {
        if self.ghost_identification_accuracy.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.ghost_identification_accuracy.iter().sum();
        sum / self.ghost_identification_accuracy.len() as f32
    }
    
    /// Get average equipment efficiency
    pub fn average_equipment_efficiency(&self) -> f32 {
        if self.equipment_efficiency.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.equipment_efficiency.iter().sum();
        sum / self.equipment_efficiency.len() as f32
    }
    
    /// Get average mission duration
    pub fn average_mission_duration(&self) -> f32 {
        if self.mission_durations.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.mission_durations.iter().sum();
        sum / self.mission_durations.len() as f32
    }
    
    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.success_rate.is_empty() {
            return 1.0; // Assume success if no data
        }
        let successes: usize = self.success_rate.iter().filter(|&&success| success).count();
        successes as f32 / self.success_rate.len() as f32
    }
}

impl AdaptiveDifficultyManager {
    /// Create a new adaptive difficulty manager
    pub fn new(base_difficulty: Difficulty) -> Self {
        Self {
            base_difficulty,
            adaptive_multiplier: 1.0,
            performance_metrics: PlayerPerformanceMetrics::new(20),
            adjustment_history: VecDeque::new(),
            enabled: true,
            min_multiplier: 0.5,
            max_multiplier: 2.0,
            adjustment_speed: 0.1,
        }
    }
    
    /// Analyze player performance and adjust difficulty if needed
    pub fn analyze_and_adjust(&mut self, time: f64) -> Option<DifficultyAdjustment> {
        if !self.enabled || self.performance_metrics.completion_times.len() < 3 {
            return None; // Need at least 3 missions for meaningful analysis
        }
        
        let adjustment = self.calculate_difficulty_adjustment(time);
        
        if let Some(adj) = &adjustment {
            let _old_multiplier = self.adaptive_multiplier;
            self.adaptive_multiplier = adj.new_multiplier;
            
            // Record the adjustment
            self.adjustment_history.push_back(adj.clone());
            if self.adjustment_history.len() > 50 {
                self.adjustment_history.pop_front();
            }
        }
        
        adjustment
    }
    
    /// Calculate what difficulty adjustment should be made
    fn calculate_difficulty_adjustment(&self, time: f64) -> Option<DifficultyAdjustment> {
        let metrics = &self.performance_metrics;
        
        // Define target ranges for optimal gameplay
        let target_completion_time = 600.0; // 10 minutes
        let target_death_rate = 0.5; // 0.5 deaths per mission on average
        let target_evidence_accuracy = 0.8; // 80% evidence accuracy
        let target_ghost_identification_accuracy = 0.7; // 70% ghost identification accuracy
        let target_equipment_efficiency = 0.6; // 60% equipment efficiency
        let target_success_rate = 0.7; // 70% mission success rate
        
        let mut adjustments = Vec::new();
        
        // Check completion time
        let avg_completion_time = metrics.average_completion_time();
        if avg_completion_time > target_completion_time * 1.5 {
            adjustments.push((0.1, AdjustmentReason::MissionTooLong));
        } else if avg_completion_time < target_completion_time * 0.5 {
            adjustments.push((-0.1, AdjustmentReason::MissionTooShort));
        }
        
        // Check death rate
        let avg_deaths = metrics.average_deaths();
        if avg_deaths > target_death_rate * 2.0 {
            adjustments.push((-0.15, AdjustmentReason::TooManyDeaths));
        } else if avg_deaths < target_death_rate * 0.3 {
            adjustments.push((0.15, AdjustmentReason::TooFewDeaths));
        }
        
        // Check evidence accuracy
        let evidence_accuracy = metrics.average_evidence_accuracy();
        if evidence_accuracy < target_evidence_accuracy * 0.7 {
            adjustments.push((-0.1, AdjustmentReason::PoorEvidenceCollection));
        } else if evidence_accuracy > target_evidence_accuracy * 1.2 {
            adjustments.push((0.1, AdjustmentReason::ExcellentEvidenceCollection));
        }
        
        // Check ghost identification accuracy
        let ghost_accuracy = metrics.average_ghost_identification_accuracy();
        if ghost_accuracy < target_ghost_identification_accuracy * 0.7 {
            adjustments.push((-0.1, AdjustmentReason::PoorGhostIdentification));
        } else if ghost_accuracy > target_ghost_identification_accuracy * 1.2 {
            adjustments.push((0.1, AdjustmentReason::ExcellentGhostIdentification));
        }
        
        // Check equipment efficiency
        let equipment_efficiency = metrics.average_equipment_efficiency();
        if equipment_efficiency < target_equipment_efficiency * 0.7 {
            adjustments.push((-0.05, AdjustmentReason::InefficientEquipmentUse));
        } else if equipment_efficiency > target_equipment_efficiency * 1.3 {
            adjustments.push((0.05, AdjustmentReason::EfficientEquipmentUse));
        }
        
        // Check success rate
        let success_rate = metrics.success_rate();
        if success_rate < target_success_rate * 0.8 {
            adjustments.push((-0.2, AdjustmentReason::PlayerStruggling));
        } else if success_rate > target_success_rate * 1.3 {
            adjustments.push((0.2, AdjustmentReason::PlayerDominating));
        }
        
        // Calculate net adjustment
        let net_adjustment: f32 = adjustments.iter().map(|(adj, _)| adj).sum();
        
        if net_adjustment.abs() < 0.05 {
            return None; // Adjustment too small
        }
        
        let new_multiplier = (self.adaptive_multiplier + net_adjustment * self.adjustment_speed)
            .clamp(self.min_multiplier, self.max_multiplier);
        
        if (new_multiplier - self.adaptive_multiplier).abs() < 0.01 {
            return None; // No meaningful change
        }
        
        let primary_reason = adjustments
            .iter()
            .max_by(|a, b| a.0.abs().partial_cmp(&b.0.abs()).unwrap())
            .map(|(_, reason)| reason.clone())
            .unwrap_or(AdjustmentReason::PlayerStruggling);
        
        let performance_data = format!(
            "Completion: {:.1}s, Deaths: {:.1}, Evidence: {:.1}%, Ghost: {:.1}%, Equipment: {:.1}%, Success: {:.1}%",
            avg_completion_time,
            avg_deaths,
            evidence_accuracy * 100.0,
            ghost_accuracy * 100.0,
            equipment_efficiency * 100.0,
            success_rate * 100.0
        );
        
        Some(DifficultyAdjustment {
            timestamp: time,
            old_multiplier: self.adaptive_multiplier,
            new_multiplier,
            reason: primary_reason,
            performance_data,
        })
    }
    
    /// Get the current effective difficulty multiplier
    pub fn get_effective_difficulty(&self) -> f32 {
        self.adaptive_multiplier
    }
    
    /// Apply adaptive difficulty to a base value
    pub fn apply_adaptive_difficulty(&self, base_value: f32) -> f32 {
        base_value * self.adaptive_multiplier
    }
    
    /// Reset adaptive difficulty to base
    pub fn reset_to_base(&mut self) {
        self.adaptive_multiplier = 1.0;
        self.performance_metrics = PlayerPerformanceMetrics::new(20);
        self.adjustment_history.clear();
    }
    
    /// Get difficulty adjustment history
    pub fn get_adjustment_history(&self) -> &VecDeque<DifficultyAdjustment> {
        &self.adjustment_history
    }
    
    /// Get performance summary
    pub fn get_performance_summary(&self) -> String {
        let metrics = &self.performance_metrics;
        format!(
            "Adaptive Difficulty: {:.1}% (Base: {:?})\n\
             Performance: {:.1}% success rate, {:.1}s avg completion, {:.1} deaths/mission\n\
             Accuracy: {:.1}% evidence, {:.1}% ghost ID, {:.1}% equipment efficiency",
            self.adaptive_multiplier * 100.0,
            self.base_difficulty,
            metrics.success_rate() * 100.0,
            metrics.average_completion_time(),
            metrics.average_deaths(),
            metrics.average_evidence_accuracy() * 100.0,
            metrics.average_ghost_identification_accuracy() * 100.0,
            metrics.average_equipment_efficiency() * 100.0
        )
    }
}

/// System that analyzes player performance and adjusts difficulty
pub fn adaptive_difficulty_system(
    mut difficulty_manager: ResMut<AdaptiveDifficultyManager>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();
    
    if let Some(adjustment) = difficulty_manager.analyze_and_adjust(current_time) {
        info!(
            "Adaptive difficulty adjustment: {:.1}% -> {:.1}% (Reason: {:?})",
            adjustment.old_multiplier * 100.0,
            adjustment.new_multiplier * 100.0,
            adjustment.reason
        );
        info!("Performance data: {}", adjustment.performance_data);
    }
}

/// System that provides difficulty feedback to the player
pub fn difficulty_feedback_system(
    difficulty_manager: Res<AdaptiveDifficultyManager>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Provide feedback every 60 seconds if adaptive difficulty is active
    if *timer >= 60.0 && difficulty_manager.enabled {
        *timer = 0.0;
        
        let summary = difficulty_manager.get_performance_summary();
        info!("Adaptive Difficulty Status:\n{}", summary);
    }
}
