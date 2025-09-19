use crate::events::walkie_types::{WalkieEvent, WalkieEventPriority, WalkieRepeatbehaviour};

impl WalkieEventPriority {
    pub fn value(&self) -> f32 {
        match self {
            WalkieEventPriority::VeryLow => 0.01,
            WalkieEventPriority::Low => 0.1,
            WalkieEventPriority::Medium => 1.0,
            WalkieEventPriority::High => 10.0,
            WalkieEventPriority::VeryHigh => 100.0,
            WalkieEventPriority::Urgent => 1000.0,
        }
    }
    pub fn time_factor(&self) -> f32 {
        match self {
            WalkieEventPriority::VeryLow => 2.0,
            WalkieEventPriority::Low => 1.3,
            WalkieEventPriority::Medium => 1.0,
            WalkieEventPriority::High => 0.9,
            WalkieEventPriority::VeryHigh => 0.8,
            WalkieEventPriority::Urgent => 0.7,
        }
    }
    pub fn is_urgent(&self) -> bool {
        matches!(self, WalkieEventPriority::Urgent)
    }
}

impl WalkieRepeatbehaviour {
    /// Returns the dice threshold for cross-mission repeat suppression
    /// Higher threshold = less likely to be suppressed (plays more often)
    /// The dice roll is compared against this threshold: if dice > threshold, message is suppressed
    pub fn dice_threshold(&self) -> u32 {
        match self {
            WalkieRepeatbehaviour::VeryLowRepeat => 1, // Very aggressive suppression (dice > 1)
            WalkieRepeatbehaviour::LowRepeat => 2,     // Aggressive suppression (dice > 2)
            WalkieRepeatbehaviour::NormalRepeat => 3,  // Current default behaviour (dice > 3)
            WalkieRepeatbehaviour::HighRepeat => 6,    // Mild suppression (dice > 6)
            WalkieRepeatbehaviour::AlwaysRepeat => 12, // Almost no suppression (dice > 12)
        }
    }

    /// Returns the minimum delay multiplier for within-mission timing
    pub fn timing_multiplier(&self) -> f64 {
        match self {
            WalkieRepeatbehaviour::VeryLowRepeat => 0.95,
            WalkieRepeatbehaviour::LowRepeat => 0.98,
            WalkieRepeatbehaviour::NormalRepeat => 1.0,
            WalkieRepeatbehaviour::HighRepeat => 1.1,
            WalkieRepeatbehaviour::AlwaysRepeat => 1.2,
        }
    }
}

impl WalkieEvent {
    pub fn time_to_play(&self, count: u32) -> f64 {
        let count = count.max(1) as f64;
        match self {
            WalkieEvent::GearInVan => 120.0 * count,
            WalkieEvent::GhostNearHunt => 120.0 * count.cbrt(),
            // WalkieEvent::MissionStartEasy => 3600.0 * 24.0 * 7.0, // Removed
            WalkieEvent::ChapterIntro(_) => 3600.0 * 24.0 * 365.0, // Effectively once (very long cooldown)
            WalkieEvent::GearExplanation(_) => 3600.0 * 24.0 * 365.0, // Effectively once (very long cooldown)

            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => 180.0 * count,
            WalkieEvent::ErraticMovementEarly => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::DoorInteractionHesitation => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::StrugglingWithGrabDrop => 180.0 * count,
            WalkieEvent::StrugglingWithHideUnhide => 180.0 * count,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => 30.0 * count,

            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => 180.0 * count,
            WalkieEvent::BreachShowcase => 9000.0 * count,
            WalkieEvent::GhostShowcase => 9000.0 * count,
            WalkieEvent::RoomLightsOnGearNeedsDark => 90.0 * count,
            WalkieEvent::ThermometerNonFreezingFixation => 120.0 * count,
            WalkieEvent::GearSelectedNotActivated => 300.0 * count,

            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => 120.0 * count,
            WalkieEvent::VeryLowSanityNoTruckReturn => 60.0 * count,
            WalkieEvent::SanityDroppedBelowThresholdDarkness => 90.0 * count,
            WalkieEvent::SanityDroppedBelowThresholdGhost => 75.0 * count,

            // --- Consumables and defence ---
            WalkieEvent::QuartzCrackedFeedback => 60.0 * count,
            WalkieEvent::QuartzShatteredFeedback => 60.0 * count,
            WalkieEvent::QuartzUnusedInRelevantSituation => 180.0 * count, // Every 3 minutes if conditions met
            WalkieEvent::SageUnusedInRelevantSituation => 180.0 * count, // Every 3 minutes if conditions met
            WalkieEvent::SageActivatedIneffectively => 180.0 * count, // Trigger every 3 minutes if conditions met
            WalkieEvent::SageUnusedDefensivelyDuringHunt => 180.0 * count, // Trigger every 3 minutes if conditions met

            // --- Ghost behaviour and Hunting ---
            WalkieEvent::PlayerStaysHiddenTooLong => 90.0 * count,

            // --- Repellent and Expulsion ---
            WalkieEvent::GhostExpelledPlayerLingers => 120.0 * count,
            WalkieEvent::HasRepellentEntersLocation => 300.0 * count,
            WalkieEvent::RepellentUsedTooFar => 60.0 * count,
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => 90.0 * count,
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => 90.0 * count,
            WalkieEvent::GhostExpelledPlayerMissed => 180.0 * count,
            WalkieEvent::DidNotSwitchStartingGearInHotspot => 180.0 * count,
            WalkieEvent::DidNotCycleToOtherGear => 180.0 * count,
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => 300.0 * count, // Trigger every 5 minutes if conditions met
            WalkieEvent::EMFNonEMF5Fixation => 120.0 * count, // Trigger every 2 minutes if conditions met
            WalkieEvent::JournalConflictingEvidence => 300.0 * count, // Trigger every 5 minutes if conditions met

            // --- Evidence Confirmation Events ---
            WalkieEvent::FreezingTempsEvidenceConfirmed => 180.0 * count,
            WalkieEvent::FloatingOrbsEvidenceConfirmed => 180.0 * count,
            WalkieEvent::UVEctoplasmEvidenceConfirmed => 180.0 * count,
            WalkieEvent::EMFLevel5EvidenceConfirmed => 180.0 * count,
            WalkieEvent::EVPEvidenceConfirmed => 180.0 * count,
            WalkieEvent::SpiritBoxEvidenceConfirmed => 180.0 * count,
            WalkieEvent::RLPresenceEvidenceConfirmed => 180.0 * count,
            WalkieEvent::CPM500EvidenceConfirmed => 180.0 * count,

            // --- Proactive Crafting Prompts ---
            WalkieEvent::PotentialGhostIDWithNewEvidence => 180.0 * count,

            // --- Mission Progression and Truck Events ---
            WalkieEvent::ClearEvidenceFoundNoActionCKey => 120.0 * count,
            WalkieEvent::ClearEvidenceFoundNoActionTruck => 120.0 * count,
            WalkieEvent::InTruckWithEvidenceNoJournal => 120.0 * count,
            WalkieEvent::HuntWarningNoPlayerEvasion => 120.0 * count,
            WalkieEvent::AllObjectivesMetReminderToEndMission => 180.0 * count,
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => 120.0 * count,
            WalkieEvent::IncorrectRepellentHint(_) => 10.0 * count,
        }
    }

    pub fn priority(&self) -> WalkieEventPriority {
        match self {
            WalkieEvent::GearInVan => WalkieEventPriority::Low,
            WalkieEvent::GhostNearHunt => WalkieEventPriority::VeryLow,
            WalkieEvent::IncorrectRepellentHint(_) => WalkieEventPriority::VeryHigh,
            WalkieEvent::ChapterIntro(_) => WalkieEventPriority::Low,
            WalkieEvent::GearExplanation(_) => WalkieEventPriority::VeryHigh,

            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => WalkieEventPriority::Medium,
            WalkieEvent::ErraticMovementEarly => WalkieEventPriority::Urgent,
            WalkieEvent::DoorInteractionHesitation => WalkieEventPriority::Medium,
            WalkieEvent::StrugglingWithGrabDrop => WalkieEventPriority::Medium,
            WalkieEvent::StrugglingWithHideUnhide => WalkieEventPriority::Medium,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => WalkieEventPriority::High,
            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => WalkieEventPriority::Medium,
            WalkieEvent::BreachShowcase => WalkieEventPriority::VeryHigh,
            WalkieEvent::GhostShowcase => WalkieEventPriority::VeryHigh,
            WalkieEvent::RoomLightsOnGearNeedsDark => WalkieEventPriority::Low,
            WalkieEvent::ThermometerNonFreezingFixation => WalkieEventPriority::Medium,
            WalkieEvent::GearSelectedNotActivated => WalkieEventPriority::Medium,
            WalkieEvent::EMFNonEMF5Fixation => WalkieEventPriority::Low,
            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => WalkieEventPriority::Medium,
            WalkieEvent::VeryLowSanityNoTruckReturn => WalkieEventPriority::VeryHigh, // Upgraded from High to VeryHigh
            WalkieEvent::SanityDroppedBelowThresholdDarkness => WalkieEventPriority::High, // Upgraded from Medium to High
            WalkieEvent::SanityDroppedBelowThresholdGhost => WalkieEventPriority::VeryHigh, // Upgraded from High to VeryHigh
            // --- Consumables and defence ---
            WalkieEvent::QuartzCrackedFeedback => WalkieEventPriority::Medium,
            WalkieEvent::QuartzShatteredFeedback => WalkieEventPriority::High,
            WalkieEvent::QuartzUnusedInRelevantSituation => WalkieEventPriority::Medium,
            WalkieEvent::SageUnusedInRelevantSituation => WalkieEventPriority::Medium,
            WalkieEvent::SageActivatedIneffectively => WalkieEventPriority::Low,
            WalkieEvent::SageUnusedDefensivelyDuringHunt => WalkieEventPriority::Medium,
            // --- Ghost behaviour and Hunting ---
            WalkieEvent::PlayerStaysHiddenTooLong => WalkieEventPriority::Low,
            // --- Repellent and Expulsion ---
            WalkieEvent::GhostExpelledPlayerLingers => WalkieEventPriority::High,
            WalkieEvent::HasRepellentEntersLocation => WalkieEventPriority::Medium,
            WalkieEvent::RepellentUsedTooFar => WalkieEventPriority::Medium,
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => WalkieEventPriority::Medium,
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => WalkieEventPriority::Medium,
            WalkieEvent::GhostExpelledPlayerMissed => WalkieEventPriority::Medium,
            WalkieEvent::DidNotSwitchStartingGearInHotspot => WalkieEventPriority::Medium,
            WalkieEvent::DidNotCycleToOtherGear => WalkieEventPriority::Medium,
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => WalkieEventPriority::Low,
            WalkieEvent::JournalConflictingEvidence => WalkieEventPriority::Medium,

            // --- Evidence Confirmation Events ---
            WalkieEvent::FreezingTempsEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::FloatingOrbsEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::UVEctoplasmEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::EMFLevel5EvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::EVPEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::SpiritBoxEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::RLPresenceEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::CPM500EvidenceConfirmed => WalkieEventPriority::VeryHigh,

            // --- Proactive Crafting Prompts ---
            WalkieEvent::PotentialGhostIDWithNewEvidence => WalkieEventPriority::VeryHigh,

            // --- Mission Progression and Truck Events ---
            WalkieEvent::ClearEvidenceFoundNoActionCKey => WalkieEventPriority::VeryLow,
            WalkieEvent::ClearEvidenceFoundNoActionTruck => WalkieEventPriority::VeryLow,
            WalkieEvent::InTruckWithEvidenceNoJournal => WalkieEventPriority::Medium,
            WalkieEvent::HuntWarningNoPlayerEvasion => WalkieEventPriority::Urgent,
            WalkieEvent::AllObjectivesMetReminderToEndMission => WalkieEventPriority::Medium,
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => WalkieEventPriority::Medium,
        }
    }

    /// Returns the repeat behaviour configuration for this event
    pub fn repeat_behaviour(&self) -> WalkieRepeatbehaviour {
        match self {
            // Introduction - this one is flavour text, which is nice to hear.
            WalkieEvent::ChapterIntro(_) => WalkieRepeatbehaviour::NormalRepeat,

            // One-time introductions and explanations - should rarely repeat
            WalkieEvent::GearExplanation(_) => WalkieRepeatbehaviour::VeryLowRepeat,

            // Basic tutorial messages - very low repeat
            WalkieEvent::ErraticMovementEarly => WalkieRepeatbehaviour::VeryLowRepeat,
            WalkieEvent::DoorInteractionHesitation => WalkieRepeatbehaviour::VeryLowRepeat,
            WalkieEvent::BreachShowcase => WalkieRepeatbehaviour::VeryLowRepeat,
            WalkieEvent::GhostShowcase => WalkieRepeatbehaviour::VeryLowRepeat,
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => {
                WalkieRepeatbehaviour::VeryLowRepeat
            }

            // Critical safety warnings - high repeat because sanity warnings are crucial for player survival
            WalkieEvent::HuntWarningNoPlayerEvasion => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::VeryLowSanityNoTruckReturn => WalkieRepeatbehaviour::HighRepeat, // Upgraded from Normal to High
            WalkieEvent::LowHealthGeneralWarning => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => WalkieRepeatbehaviour::NormalRepeat,

            // Evidence confirmations - should repeat often for feedback
            WalkieEvent::FreezingTempsEvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::FloatingOrbsEvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::UVEctoplasmEvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::EMFLevel5EvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::EVPEvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::SpiritBoxEvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::RLPresenceEvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::CPM500EvidenceConfirmed => WalkieRepeatbehaviour::HighRepeat,

            // Incorrect repellent hints are critical for gameplay - always repeat
            WalkieEvent::IncorrectRepellentHint(_) => WalkieRepeatbehaviour::AlwaysRepeat,

            // Critical feedback messages - normal repeat
            WalkieEvent::GhostNearHunt => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::QuartzCrackedFeedback => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::QuartzShatteredFeedback => WalkieRepeatbehaviour::NormalRepeat,

            // Important gameplay hints - should repeat often to help with crafting decisions
            WalkieEvent::JournalPointsToOneGhostNoCraft => WalkieRepeatbehaviour::HighRepeat,

            // Useful information - should repeat a lot
            WalkieEvent::AllObjectivesMetReminderToEndMission => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::GhostExpelledPlayerLingers => WalkieRepeatbehaviour::HighRepeat,
            WalkieEvent::PotentialGhostIDWithNewEvidence => WalkieRepeatbehaviour::AlwaysRepeat,
            WalkieEvent::GhostExpelledPlayerMissed => WalkieRepeatbehaviour::AlwaysRepeat,

            // Contextual hints and reminders - those that might fire even when the player knows what they're doing should not repeat that much
            WalkieEvent::StrugglingWithGrabDrop => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::StrugglingWithHideUnhide => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::DarkRoomNoLightUsed => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::ThermometerNonFreezingFixation => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::EMFNonEMF5Fixation => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::DidNotSwitchStartingGearInHotspot => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::DidNotCycleToOtherGear => WalkieRepeatbehaviour::LowRepeat,

            // Contextual hints and reminders - high repeat for sanity warnings as they're critical
            WalkieEvent::PlayerStuckAtStart => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::GearSelectedNotActivated => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::RoomLightsOnGearNeedsDark => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::SanityDroppedBelowThresholdDarkness => WalkieRepeatbehaviour::HighRepeat, // Upgraded from Normal to High
            WalkieEvent::SanityDroppedBelowThresholdGhost => WalkieRepeatbehaviour::HighRepeat, // Upgraded from Normal to High
            WalkieEvent::HasRepellentEntersLocation => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::RepellentUsedTooFar => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => {
                WalkieRepeatbehaviour::NormalRepeat
            }
            WalkieEvent::JournalConflictingEvidence => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::InTruckWithEvidenceNoJournal => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::QuartzUnusedInRelevantSituation => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::SageUnusedInRelevantSituation => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::SageActivatedIneffectively => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::SageUnusedDefensivelyDuringHunt => WalkieRepeatbehaviour::NormalRepeat,
            WalkieEvent::PlayerStaysHiddenTooLong => WalkieRepeatbehaviour::NormalRepeat,

            // Low priority hints - can be suppressed more
            WalkieEvent::GearInVan => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::ClearEvidenceFoundNoActionCKey => WalkieRepeatbehaviour::LowRepeat,
            WalkieEvent::ClearEvidenceFoundNoActionTruck => WalkieRepeatbehaviour::LowRepeat,
        }
    }

    /// Calculate the effective priority for this event, taking into account how many times
    /// it has been played in previous missions. Events that have been played many times
    /// will have their priority downgraded to give fresh content higher precedence.
    pub fn effective_priority(&self, previous_mission_play_count: u32) -> WalkieEventPriority {
        let base_priority = self.priority();
        let repeat_behaviour = self.repeat_behaviour();

        // Events with AlwaysRepeat behaviour should not be downgraded
        if matches!(repeat_behaviour, WalkieRepeatbehaviour::AlwaysRepeat) {
            return base_priority;
        }

        // Calculate downgrade amount based on play count and repeat behaviour
        let downgrade_factor = match repeat_behaviour {
            WalkieRepeatbehaviour::VeryLowRepeat => {
                // Aggressive downgrading for one-time events
                match previous_mission_play_count {
                    0 => 0,
                    _ => 4,
                }
            }
            WalkieRepeatbehaviour::LowRepeat => {
                // Moderate downgrading for tutorial events
                match previous_mission_play_count {
                    0 => 0,
                    1 => 1,     // Drop 1 priority level after first play
                    2..=3 => 2, // Drop 2 priority levels after 2-3 plays
                    _ => 3,     // Drop 3 priority levels after many plays
                }
            }
            WalkieRepeatbehaviour::NormalRepeat => {
                // Standard downgrading for normal events
                match previous_mission_play_count {
                    0..=1 => 0,
                    2..=4 => 1, // Drop 1 priority level after several plays
                    5..=9 => 2, // Drop 2 priority levels after many plays
                    _ => 3,     // Drop 3 priority levels after excessive plays
                }
            }
            WalkieRepeatbehaviour::HighRepeat => {
                // Minimal downgrading for feedback events
                match previous_mission_play_count {
                    0..=3 => 0,
                    4..=9 => 1, // Drop 1 priority level after many plays
                    _ => 2,     // Drop 2 priority levels after excessive plays
                }
            }
            WalkieRepeatbehaviour::AlwaysRepeat => 0, // Already handled above
        };

        // Apply the downgrade
        match base_priority {
            WalkieEventPriority::Urgent => match downgrade_factor {
                0 => WalkieEventPriority::Urgent,
                1 => WalkieEventPriority::VeryHigh,
                2 => WalkieEventPriority::High,
                3 => WalkieEventPriority::Medium,
                _ => WalkieEventPriority::Low,
            },
            WalkieEventPriority::VeryHigh => match downgrade_factor {
                0 => WalkieEventPriority::VeryHigh,
                1 => WalkieEventPriority::High,
                2 => WalkieEventPriority::Medium,
                3 => WalkieEventPriority::Low,
                _ => WalkieEventPriority::VeryLow,
            },
            WalkieEventPriority::High => match downgrade_factor {
                0 => WalkieEventPriority::High,
                1 => WalkieEventPriority::Medium,
                2 => WalkieEventPriority::Low,
                _ => WalkieEventPriority::VeryLow,
            },
            WalkieEventPriority::Medium => match downgrade_factor {
                0 => WalkieEventPriority::Medium,
                1 => WalkieEventPriority::Low,
                _ => WalkieEventPriority::VeryLow,
            },
            WalkieEventPriority::Low => match downgrade_factor {
                0 => WalkieEventPriority::Low,
                _ => WalkieEventPriority::VeryLow,
            },
            WalkieEventPriority::VeryLow => WalkieEventPriority::VeryLow,
        }
    }
}
