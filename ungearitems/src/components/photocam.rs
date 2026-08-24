use bevy::prelude::*;
use rand::Rng as _;
use uncore::random_seed;
use uncore::{
    components::board::position::Position,
    types::gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID},
};
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;

use super::{Gear, GearKind, on_off};

/// Camera that can capture paranormal anomalies on film. Photos of the ghost
/// are rare and prized by the agency.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Photocam {
    pub enabled: bool,
    /// Time left showing the flash animation.
    pub flash_secs_left: f32,
    /// Time left showing the verdict of the last shot.
    pub verdict_secs_left: f32,
    /// Outcome of the last shot: Some(true) = anomaly, Some(false) = nothing.
    pub last_shot_anomaly: Option<bool>,
    pub photos_taken: u32,
    pub anomalies_captured: u32,
    /// Last known position of the tool, cached from `update()` so the shutter
    /// trigger can evaluate the scene.
    last_pos: Option<Position>,
}

impl Photocam {
    /// Rolls whether a shot captured something paranormal. Ghosts close by,
    /// in well lit scenes, are much more likely to show up.
    fn evaluate_shot(&self, gs: &GearStuff, pos: &Position) -> bool {
        let mut rng = random_seed::rng();
        let alpha = gs.bf.ghost_dynamics.visual_alpha_multiplier;
        for gp in &gs.bf.ghost_positions {
            if gp.z.round() != pos.z.round() {
                continue;
            }
            let dist = gp.distance(pos);
            if dist > 10.0 {
                continue;
            }
            // Chance peaks near 60% point-blank and fades to ~5% at range.
            let chance = (1.8 / (dist + 1.0)).min(0.6) * 0.65 * alpha.clamp(0.2, 2.0);
            if rng.random_bool(chance.clamp(0.03, 0.75).into()) {
                return true;
            }
        }
        false
    }

    fn take_photo(&mut self, gs: &mut GearStuff, pos: &Position) {
        self.photos_taken += 1;
        self.flash_secs_left = 0.25;
        self.verdict_secs_left = 3.0;
        let anomaly = self.evaluate_shot(gs, pos);
        self.last_shot_anomaly = Some(anomaly);
        if anomaly {
            self.anomalies_captured += 1;
            gs.play_audio("sounds/effects-chirp-high.ogg".into(), 0.5, pos);
        } else {
            gs.play_audio("sounds/effects-chirp-shorter.ogg".into(), 0.35, pos);
        }
    }
}

impl GearUsable for Photocam {
    fn can_enable(&self) -> bool {
        self.flash_secs_left <= 0.0
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.flash_secs_left > 0.125 {
            GearSpriteID::PhotocamFlash1
        } else if self.flash_secs_left > 0.0 {
            GearSpriteID::PhotocamFlash2
        } else {
            GearSpriteID::Photocam
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Photocam"
    }

    fn get_description(&self) -> &'static str {
        "Takes photos, hopefully of something paranormal."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.is_enabled() {
            if self.verdict_secs_left > 0.0 {
                match self.last_shot_anomaly {
                    Some(true) => "!ANOMALY ON FILM!".to_string(),
                    Some(false) => "Nothing unusual.".to_string(),
                    None => String::new(),
                }
            } else {
                format!(
                    "Shots: {}\nAnomalies: {}",
                    self.photos_taken, self.anomalies_captured
                )
            }
        } else {
            String::new()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        if !self.enabled || !self.can_enable() {
            return;
        }
        let pos = self.last_pos.unwrap_or(gs.bf.breach_pos);
        self.take_photo(gs, &pos);
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        self.last_pos = Some(*pos);
        if self.flash_secs_left > 0.0 {
            self.flash_secs_left -= gs.time.delta_secs();
        }
        if self.verdict_secs_left > 0.0 && !self.is_enabled() {
            self.verdict_secs_left = 0.0;
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn is_electronic(&self) -> bool {
        true
    }

    fn apply_electromagnetic_interference(&mut self, warning_level: f32, distance2: f32) {
        if warning_level < 0.0001 || !self.is_enabled() {
            return;
        }
        let mut rng = random_seed::rng();
        let effect_strength = warning_level * (100.0 / distance2).min(1.0);
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) * 0.5 {
            // The flash misfires during strong interference.
            self.flash_secs_left = 0.25;
        }
    }
}

impl From<Photocam> for Gear {
    fn from(value: Photocam) -> Self {
        Gear::new_from_kind(GearKind::Photocam, value.box_clone())
    }
}
