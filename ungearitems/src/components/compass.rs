use bevy::prelude::*;
use rand::Rng as _;
use uncore::random_seed;
use uncore::{
    components::board::position::Position,
    types::gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID},
};
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;

use super::{Gear, GearKind};

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Compass {
    pub enabled: bool,
    /// Smoothed displayed bearing in degrees (0 = North).
    pub needle_bearing_deg: f32,
}

/// Cardinal direction label for a bearing in degrees.
fn cardinal(bearing_deg: f32) -> &'static str {
    const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let idx = ((bearing_deg.rem_euclid(360.0) + 22.5) / 45.0) as usize % 8;
    DIRS[idx]
}

/// Shortest signed angle difference in degrees (-180..180).
fn angle_diff(from: f32, to: f32) -> f32 {
    (to - from + 180.0).rem_euclid(360.0) - 180.0
}

impl Compass {
    /// Signal strength 0..=1 of the closest ghost, if any is near enough to
    /// deflect the needle.
    fn ghost_influence(&self, gs: &GearStuff, pos: &Position) -> (f32, f32) {
        // Returns (strength 0..1, bearing degrees toward the anomaly source)
        let mut best: Option<(f32, f32)> = None;
        for gp in &gs.bf.ghost_positions {
            let dz = (gp.z.round() - pos.z.round()).abs();
            let dx = gp.x - pos.x;
            let dy = gp.y - pos.y;
            let dist = (dx * dx + dy * dy + dz * dz * 100.0).sqrt();
            if dist > 14.0 {
                continue;
            }
            let strength = 1.0 - dist / 14.0;
            // Bearing: 0 deg = North (-Y), clockwise towards East (+X).
            let bearing = dy.atan2(dx).to_degrees() + 90.0;
            if best.is_none_or(|(s, _)| strength > s) {
                best = Some((strength, bearing));
            }
        }
        best.unwrap_or((0.0, 0.0))
    }
}

impl GearUsable for Compass {
    fn can_enable(&self) -> bool {
        true
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        // The compass has a single sprite; the needle wobble is conveyed by the
        // status text.
        GearSpriteID::Compass
    }

    fn get_display_name(&self) -> &'static str {
        "Compass"
    }

    fn get_description(&self) -> &'static str {
        "Measures the Earth's magnetic field, and sometimes the ghost."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        if !self.enabled {
            return format!("{name}: off");
        }
        let dir = cardinal(self.needle_bearing_deg);
        format!("{name}: on\nBearing: {dir}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        self.enabled = !self.enabled;
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        if !self.enabled {
            return;
        }
        let mut rng = random_seed::rng();
        let dt = gs.time.delta_secs();
        let (influence, target_bearing) = self.ghost_influence(gs, pos);

        // Far from any anomaly the needle lazily wanders around true north;
        // close to one it locks onto it and trembles.
        let wander_noise: f32 = rng.random_range(-1.0..1.0) * 20.0 * (1.0 - influence);
        let tremble: f32 = rng.random_range(-1.0..1.0) * 6.0 * influence;
        let target = if influence > 0.0 {
            target_bearing + tremble
        } else {
            wander_noise
        };

        let diff = angle_diff(self.needle_bearing_deg, target);
        let step = diff * (dt * (2.5 + 6.0 * influence)).clamp(0.05, 1.0);
        self.needle_bearing_deg += step;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Compass> for Gear {
    fn from(value: Compass) -> Self {
        Gear::new_from_kind(GearKind::Compass, value.box_clone())
    }
}
