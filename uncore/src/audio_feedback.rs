//! ## Audio Feedback Module
//!
//! This module provides enhanced audio feedback for evidence gathering,
//! equipment usage, and gameplay events to improve player experience.

use bevy::prelude::*;
use crate::types::evidence::Evidence;

/// Resource for managing audio feedback events
#[derive(Resource, Default)]
pub struct AudioFeedbackManager {
    pub pending_feedback: Vec<AudioFeedbackEvent>,
}

/// Events that trigger audio feedback
#[derive(Debug, Clone)]
pub enum AudioFeedbackEvent {
    /// Evidence was confirmed/collected
    EvidenceConfirmed(Evidence),
    /// Equipment threshold reached (e.g., thermometer reading)
    EquipmentThreshold {
        equipment: EquipmentType,
        threshold: ThresholdType,
        value: f32,
    },
    /// Equipment activated/deactivated
    EquipmentToggle {
        equipment: EquipmentType,
        active: bool,
    },
    /// Mission progress milestone
    MissionProgress(MissionMilestone),
    /// Ghost interaction event
    GhostInteraction(GhostEvent),
}

/// Types of equipment that can provide feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentType {
    Thermometer,
    EMFMeter,
    SpiritBox,
    UVTorch,
    RedTorch,
    VideoCamera,
    Recorder,
    GeigerCounter,
}

/// Types of thresholds for equipment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdType {
    Low,
    High,
    Critical,
    Optimal,
}

/// Mission progress milestones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionMilestone {
    FirstEvidence,
    HalfEvidence,
    AllEvidence,
    GhostIdentified,
    MissionCompleted,
}

/// Ghost interaction events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostEvent {
    BreachDetected,
    GhostSpotted,
    HuntStarted,
    HuntEnded,
    GhostBanished,
}

impl AudioFeedbackManager {
    /// Add a new feedback event to the queue
    pub fn add_feedback(&mut self, event: AudioFeedbackEvent) {
        self.pending_feedback.push(event);
        
        // Limit queue size to prevent memory issues
        if self.pending_feedback.len() > 50 {
            self.pending_feedback.drain(0..25);
        }
    }
    
    
    /// Clear all pending feedback
    pub fn clear(&mut self) {
        self.pending_feedback.clear();
    }
    
    /// Get the number of pending feedback events
    pub fn pending_count(&self) -> usize {
        self.pending_feedback.len()
    }
}

/// System that processes audio feedback events
pub fn process_audio_feedback(
    mut feedback_manager: ResMut<AudioFeedbackManager>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    audio_settings: Res<bevy_persistent::Persistent<unsettings::audio::AudioSettings>>,
) {
    // Process pending feedback events
    let events = feedback_manager.pending_feedback.drain(..).collect::<Vec<_>>();
    
    for event in events {
        let sound_path = get_sound_path_for_event(&event);
        if let Some(sound_path) = sound_path {
            // Check if audio is enabled and volume is sufficient
            if audio_settings.volume_master.as_f32() > 0.0 && audio_settings.volume_effects.as_f32() > 0.0 {
                let audio_source = asset_server.load(sound_path);
                
                // Spawn audio player with appropriate volume
                commands.spawn(AudioPlayer::new(audio_source))
                    .insert(bevy::audio::PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::Linear(
                            audio_settings.volume_master.as_f32() * audio_settings.volume_effects.as_f32()
                        ),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                        spatial_scale: None,
                        ..default()
                    });
                
                info!("Playing audio feedback: {} for event: {:?}", sound_path, event);
            }
        }
    }
}

/// Helper function to get sound path for an event
fn get_sound_path_for_event(event: &AudioFeedbackEvent) -> Option<&'static str> {
    match event {
        AudioFeedbackEvent::EvidenceConfirmed(evidence) => {
            match evidence {
                Evidence::FreezingTemp => Some("sounds/evidence_freezing.ogg"),
                Evidence::FloatingOrbs => Some("sounds/evidence_orbs.ogg"),
                Evidence::UVEctoplasm => Some("sounds/evidence_ectoplasm.ogg"),
                Evidence::EMFLevel5 => Some("sounds/evidence_emf.ogg"),
                Evidence::EVPRecording => Some("sounds/evidence_evp.ogg"),
                Evidence::SpiritBox => Some("sounds/evidence_spiritbox.ogg"),
                Evidence::RLPresence => Some("sounds/evidence_presence.ogg"),
                Evidence::CPM500 => Some("sounds/evidence_radiation.ogg"),
            }
        }
        AudioFeedbackEvent::EquipmentThreshold { equipment, threshold, .. } => {
            match (equipment, threshold) {
                (EquipmentType::Thermometer, ThresholdType::Low) => Some("sounds/thermometer_cold.ogg"),
                (EquipmentType::Thermometer, ThresholdType::Critical) => Some("sounds/thermometer_freezing.ogg"),
                (EquipmentType::EMFMeter, ThresholdType::High) => Some("sounds/emf_spike.ogg"),
                (EquipmentType::EMFMeter, ThresholdType::Critical) => Some("sounds/emf_level5.ogg"),
                (EquipmentType::GeigerCounter, ThresholdType::High) => Some("sounds/geiger_high.ogg"),
                (EquipmentType::GeigerCounter, ThresholdType::Critical) => Some("sounds/geiger_critical.ogg"),
                _ => None,
            }
        }
        AudioFeedbackEvent::EquipmentToggle { equipment, active } => {
            match (equipment, active) {
                (EquipmentType::Thermometer, true) => Some("sounds/thermometer_on.ogg"),
                (EquipmentType::Thermometer, false) => Some("sounds/thermometer_off.ogg"),
                (EquipmentType::EMFMeter, true) => Some("sounds/emf_on.ogg"),
                (EquipmentType::EMFMeter, false) => Some("sounds/emf_off.ogg"),
                (EquipmentType::SpiritBox, true) => Some("sounds/spiritbox_on.ogg"),
                (EquipmentType::SpiritBox, false) => Some("sounds/spiritbox_off.ogg"),
                _ => None,
            }
        }
        AudioFeedbackEvent::MissionProgress(milestone) => {
            match milestone {
                MissionMilestone::FirstEvidence => Some("sounds/milestone_first_evidence.ogg"),
                MissionMilestone::HalfEvidence => Some("sounds/milestone_half_evidence.ogg"),
                MissionMilestone::AllEvidence => Some("sounds/milestone_all_evidence.ogg"),
                MissionMilestone::GhostIdentified => Some("sounds/milestone_ghost_identified.ogg"),
                MissionMilestone::MissionCompleted => Some("sounds/milestone_completed.ogg"),
            }
        }
        AudioFeedbackEvent::GhostInteraction(ghost_event) => {
            match ghost_event {
                GhostEvent::BreachDetected => Some("sounds/ghost_breach_detected.ogg"),
                GhostEvent::GhostSpotted => Some("sounds/ghost_spotted.ogg"),
                GhostEvent::HuntStarted => Some("sounds/ghost_hunt_start.ogg"),
                GhostEvent::HuntEnded => Some("sounds/ghost_hunt_end.ogg"),
                GhostEvent::GhostBanished => Some("sounds/ghost_banished.ogg"),
            }
        }
    }
}

/// Helper function to trigger evidence confirmation feedback
pub fn trigger_evidence_feedback(
    feedback_manager: &mut ResMut<AudioFeedbackManager>,
    evidence: Evidence,
) {
    feedback_manager.add_feedback(AudioFeedbackEvent::EvidenceConfirmed(evidence));
}

/// Helper function to trigger equipment threshold feedback
pub fn trigger_equipment_threshold_feedback(
    feedback_manager: &mut ResMut<AudioFeedbackManager>,
    equipment: EquipmentType,
    threshold: ThresholdType,
    value: f32,
) {
    feedback_manager.add_feedback(AudioFeedbackEvent::EquipmentThreshold {
        equipment,
        threshold,
        value,
    });
}

/// Helper function to trigger mission progress feedback
pub fn trigger_mission_progress_feedback(
    feedback_manager: &mut ResMut<AudioFeedbackManager>,
    milestone: MissionMilestone,
) {
    feedback_manager.add_feedback(AudioFeedbackEvent::MissionProgress(milestone));
}
