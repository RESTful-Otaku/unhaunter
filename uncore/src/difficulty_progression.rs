//! ## Difficulty Progression System
//!
//! This module provides smooth difficulty progression curves and
//! better balancing between difficulty levels.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::difficulty::Difficulty;

/// Enhanced difficulty structure with smoother progression curves
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDifficultySettings {
    /// Base difficulty level
    pub base_difficulty: Difficulty,
    /// Difficulty progression multiplier (0.0 to 1.0, where 1.0 is full difficulty)
    pub progression_multiplier: f32,
    /// Player experience level (affects available equipment and hints)
    pub experience_level: u32,
    /// Unlocked difficulty levels
    pub unlocked_difficulties: Vec<Difficulty>,
    /// Progression curve settings
    pub progression_curve: ProgressionCurve,
}

/// Settings for difficulty progression curves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionCurve {
    /// Curve type for difficulty progression
    pub curve_type: CurveType,
    /// Smoothing factor for transitions (0.0 = instant, 1.0 = very smooth)
    pub smoothing_factor: f32,
    /// Whether to use exponential scaling
    pub use_exponential: bool,
    /// Base scaling factor
    pub base_scaling: f32,
}

/// Types of progression curves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Logarithmic,
    Exponential,
    SCurve, // Smooth S-curve for gradual difficulty increase
    Custom(Vec<f32>), // Custom curve points
}

impl Default for EnhancedDifficultySettings {
    fn default() -> Self {
        Self {
            base_difficulty: Difficulty::TutorialChapter1,
            progression_multiplier: 0.0,
            experience_level: 1,
            unlocked_difficulties: vec![Difficulty::TutorialChapter1],
            progression_curve: ProgressionCurve::default(),
        }
    }
}

impl Default for ProgressionCurve {
    fn default() -> Self {
        Self {
            curve_type: CurveType::SCurve,
            smoothing_factor: 0.8,
            use_exponential: false,
            base_scaling: 1.0,
        }
    }
}

impl EnhancedDifficultySettings {
    /// Create new enhanced difficulty settings
    pub fn new(base_difficulty: Difficulty) -> Self {
        Self {
            base_difficulty,
            progression_multiplier: 0.0,
            experience_level: 1,
            unlocked_difficulties: vec![base_difficulty],
            progression_curve: ProgressionCurve::default(),
        }
    }
    
    /// Get the effective difficulty multiplier based on progression
    pub fn get_effective_multiplier(&self) -> f32 {
        self.apply_progression_curve(self.progression_multiplier)
    }
    
    /// Apply progression curve to a raw multiplier
    fn apply_progression_curve(&self, raw_multiplier: f32) -> f32 {
        let curve = &self.progression_curve;
        
        match curve.curve_type {
            CurveType::Linear => raw_multiplier,
            CurveType::Logarithmic => {
                if raw_multiplier <= 0.0 {
                    0.0
                } else {
                    (raw_multiplier.ln() + 1.0) / (2.0_f32.ln())
                }
            }
            CurveType::Exponential => {
                if curve.use_exponential {
                    (raw_multiplier * curve.base_scaling).exp() - 1.0
                } else {
                    raw_multiplier * raw_multiplier
                }
            }
            CurveType::SCurve => {
                // Smooth S-curve: starts slow, accelerates in middle, slows at end
                let x = raw_multiplier.clamp(0.0, 1.0);
                let smooth = 3.0 * x * x - 2.0 * x * x * x; // Cubic hermite interpolation
                smooth * curve.base_scaling
            }
            CurveType::Custom(ref points) => {
                if points.is_empty() {
                    return raw_multiplier;
                }
                
                let scaled_x = raw_multiplier * (points.len() - 1) as f32;
                let index = scaled_x.floor() as usize;
                let t = scaled_x - index as f32;
                
                if index >= points.len() - 1 {
                    *points.last().unwrap()
                } else {
                    // Linear interpolation between points
                    points[index] * (1.0 - t) + points[index + 1] * t
                }
            }
        }
    }
    
    /// Progress to the next difficulty level
    pub fn progress_to_next_difficulty(&mut self) -> bool {
        let current_index = self.get_difficulty_index(self.base_difficulty);
        let next_difficulty = self.get_next_available_difficulty(current_index + 1);
        
        if let Some(next) = next_difficulty {
            if !self.unlocked_difficulties.contains(&next) {
                self.unlocked_difficulties.push(next);
            }
            self.base_difficulty = next;
            self.progression_multiplier = 0.0; // Reset progression for new difficulty
            return true;
        }
        
        false
    }
    
    /// Get the next available difficulty level
    fn get_next_available_difficulty(&self, start_index: usize) -> Option<Difficulty> {
        let all_difficulties = Difficulty::all().collect::<Vec<_>>();
        
        for i in start_index..all_difficulties.len() {
            let difficulty = all_difficulties[i];
            if !self.unlocked_difficulties.contains(&difficulty) {
                return Some(difficulty);
            }
        }
        
        None
    }
    
    /// Get the index of a difficulty level
    fn get_difficulty_index(&self, difficulty: Difficulty) -> usize {
        Difficulty::all().position(|d| d == difficulty).unwrap_or(0)
    }
    
    /// Increase experience level
    pub fn increase_experience(&mut self) {
        self.experience_level += 1;
        
        // Unlock new difficulties based on experience
        self.unlock_difficulties_for_experience();
    }
    
    /// Unlock difficulties based on experience level
    fn unlock_difficulties_for_experience(&mut self) {
        let all_difficulties = Difficulty::all().collect::<Vec<_>>();
        let unlock_thresholds = vec![1, 3, 5, 7, 9, 12, 15, 18, 20]; // Experience levels to unlock difficulties
        
        for (i, &threshold) in unlock_thresholds.iter().enumerate() {
            if self.experience_level >= threshold && i < all_difficulties.len() {
                let difficulty = all_difficulties[i];
                if !self.unlocked_difficulties.contains(&difficulty) {
                    self.unlocked_difficulties.push(difficulty);
                }
            }
        }
    }
    
    /// Get available hints based on experience level
    pub fn get_available_hints(&self) -> HintAvailability {
        match self.experience_level {
            1..=2 => HintAvailability::Full, // New players get full hints
            3..=5 => HintAvailability::Partial, // Some hints
            6..=10 => HintAvailability::Minimal, // Few hints
            _ => HintAvailability::None, // Experienced players get no hints
        }
    }
    
    /// Get equipment availability based on experience level
    pub fn get_available_equipment(&self) -> EquipmentAvailability {
        match self.experience_level {
            1..=2 => EquipmentAvailability::Basic, // Basic equipment only
            3..=5 => EquipmentAvailability::Standard, // Standard equipment
            6..=10 => EquipmentAvailability::Advanced, // Advanced equipment
            _ => EquipmentAvailability::All, // All equipment available
        }
    }
    
    /// Check if a difficulty level is unlocked
    pub fn is_difficulty_unlocked(&self, difficulty: Difficulty) -> bool {
        self.unlocked_difficulties.contains(&difficulty)
    }
    
    /// Get progression status string
    pub fn get_progression_status(&self) -> String {
        format!(
            "Difficulty: {:?} ({:.1}% progression)\n\
             Experience Level: {}\n\
             Unlocked Difficulties: {}/{}",
            self.base_difficulty,
            self.progression_multiplier * 100.0,
            self.experience_level,
            self.unlocked_difficulties.len(),
            Difficulty::all().count()
        )
    }
}

/// Hint availability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintAvailability {
    Full,    // All hints available
    Partial, // Some hints available
    Minimal, // Few hints available
    None,    // No hints available
}

/// Equipment availability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentAvailability {
    Basic,    // Basic equipment only
    Standard, // Standard equipment
    Advanced, // Advanced equipment
    All,      // All equipment available
}

/// System that manages difficulty progression
pub fn difficulty_progression_system(
    mut difficulty_settings: ResMut<EnhancedDifficultySettings>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Update progression every 30 seconds
    if *timer >= 30.0 {
        *timer = 0.0;
        
        // Gradually increase progression multiplier
        difficulty_settings.progression_multiplier = (difficulty_settings.progression_multiplier + 0.01)
            .min(1.0);
        
        // Check if ready to progress to next difficulty
        if difficulty_settings.progression_multiplier >= 1.0 {
            if difficulty_settings.progress_to_next_difficulty() {
                info!("Progressed to new difficulty level: {:?}", difficulty_settings.base_difficulty);
            }
        }
    }
}

/// System that provides progression feedback
pub fn progression_feedback_system(
    difficulty_settings: Res<EnhancedDifficultySettings>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Provide feedback every 120 seconds
    if *timer >= 120.0 {
        *timer = 0.0;
        
        let status = difficulty_settings.get_progression_status();
        info!("Difficulty Progression Status:\n{}", status);
    }
}

/// Helper functions for difficulty balancing
pub mod difficulty_balancing {
    use super::*;
    
    /// Calculate balanced ghost speed based on difficulty and progression
    pub fn calculate_balanced_ghost_speed(
        base_difficulty: Difficulty,
        progression_multiplier: f32,
    ) -> f32 {
        let base_speed = base_difficulty.ghost_speed();
        let progression_curve = ProgressionCurve::default();
        let effective_multiplier = progression_curve.apply_progression_curve(progression_multiplier);
        
        // Apply smooth scaling
        base_speed * (1.0 + effective_multiplier * 0.5)
    }
    
    /// Calculate balanced equipment sensitivity based on difficulty
    pub fn calculate_balanced_equipment_sensitivity(
        base_difficulty: Difficulty,
        experience_level: u32,
    ) -> f32 {
        let base_sensitivity = base_difficulty.equipment_sensitivity();
        
        // Reduce sensitivity for experienced players
        let experience_factor = 1.0 - (experience_level as f32 * 0.05).min(0.3);
        
        base_sensitivity * experience_factor
    }
    
    /// Calculate balanced evidence clarity based on difficulty and progression
    pub fn calculate_balanced_evidence_clarity(
        base_difficulty: Difficulty,
        progression_multiplier: f32,
    ) -> f32 {
        // Use equipment sensitivity as a proxy for evidence clarity
        let base_clarity = base_difficulty.equipment_sensitivity();
        let progression_curve = ProgressionCurve::default();
        let effective_multiplier = progression_curve.apply_progression_curve(progression_multiplier);
        
        // Evidence becomes less clear as difficulty increases
        base_clarity * (1.0 - effective_multiplier * 0.3)
    }
}

impl ProgressionCurve {
    /// Apply progression curve to a value
    pub fn apply_progression_curve(&self, raw_value: f32) -> f32 {
        let curve = self;
        
        match curve.curve_type {
            CurveType::Linear => raw_value,
            CurveType::Logarithmic => {
                if raw_value <= 0.0 {
                    0.0
                } else {
                    (raw_value.ln() + 1.0) / (2.0_f32.ln())
                }
            }
            CurveType::Exponential => {
                if curve.use_exponential {
                    (raw_value * curve.base_scaling).exp() - 1.0
                } else {
                    raw_value * raw_value
                }
            }
            CurveType::SCurve => {
                let x = raw_value.clamp(0.0, 1.0);
                let smooth = 3.0 * x * x - 2.0 * x * x * x;
                smooth * curve.base_scaling
            }
            CurveType::Custom(ref points) => {
                if points.is_empty() {
                    return raw_value;
                }
                
                let scaled_x = raw_value * (points.len() - 1) as f32;
                let index = scaled_x.floor() as usize;
                let t = scaled_x - index as f32;
                
                if index >= points.len() - 1 {
                    *points.last().unwrap()
                } else {
                    points[index] * (1.0 - t) + points[index + 1] * t
                }
            }
        }
    }
}
