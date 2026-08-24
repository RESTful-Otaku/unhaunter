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

/// Measures atmospheric static electricity. Ghosts charging up a hunt build a
/// measurable electrostatic field before they strike.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct EStaticMeter {
    pub enabled: bool,
    /// Smoothed displayed reading (V/m).
    pub display_v: f32,
    /// Time left on the current discharge alarm.
    pub alarm_secs_left: f32,
}

impl EStaticMeter {
    /// Static field strength in V/m at the tool position.
    fn measure(&self, gs: &GearStuff, pos: &Position) -> f32 {
        let mut rng = random_seed::rng();
        // Ambient static + sensor noise.
        let mut v = 110.0 + rng.random_range(-15.0..15.0);

        // The breach leaks charge into the building; closer rooms read higher.
        let d2_breach = gs.bf.breach_pos.distance2(pos) + 20.0;
        v += 4000.0 / d2_breach;

        // An angered ghost builds up a big discharge before hunting.
        if let Some(wpos) = &gs.bf.ghost_warning_position {
            let d2 = pos.distance2(wpos) + 10.0;
            v += gs.bf.ghost_warning_intensity * 9000.0 / d2;
        }
        v
    }
}

impl GearUsable for EStaticMeter {
    fn can_enable(&self) -> bool {
        self.alarm_secs_left <= 0.0
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        // Single sprite for this device.
        GearSpriteID::EStaticMeter
    }

    fn get_display_name(&self) -> &'static str {
        "E-Static Meter"
    }

    fn get_description(&self) -> &'static str {
        "Measures static electricity in the air. Might warn if the ghost is angering."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.is_enabled() {
            if self.alarm_secs_left > 0.0 {
                format!("{:.0} V/m\nDISCHARGE WARNING", self.display_v)
            } else {
                format!("Reading: {:.0} V/m", self.display_v)
            }
        } else {
            String::new()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        if self.enabled {
            self.enabled = false;
        } else if self.can_enable() {
            self.enabled = true;
        }
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let dt = gs.time.delta_secs();
        if self.alarm_secs_left > 0.0 {
            self.alarm_secs_left -= dt;
        }
        if !self.enabled {
            return;
        }

        let target = self.measure(gs, pos);
        // Fast attack, slow decay for an analog feel.
        let k = if target > self.display_v { 8.0 } else { 1.5 };
        self.display_v += (target - self.display_v) * (dt * k).min(1.0);

        const ALARM_THRESHOLD: f32 = 700.0;
        if self.alarm_secs_left <= 0.0 && self.display_v > ALARM_THRESHOLD {
            self.alarm_secs_left = 1.5;
            gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.35, pos);
        }

        // Electronic interference during hunt warnings.
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn is_electronic(&self) -> bool {
        true
    }

    fn apply_electromagnetic_interference(&mut self, warning_level: f32, distance2: f32) {
        if warning_level < 0.0001 || !self.enabled {
            return;
        }
        let mut rng = random_seed::rng();
        let effect_strength = warning_level * (100.0 / distance2).min(1.0);
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
            self.display_v += rng.random_range(-200.0..600.0);
            self.alarm_secs_left = self.alarm_secs_left.max(0.4);
        }
    }
}

impl From<EStaticMeter> for Gear {
    fn from(value: EStaticMeter) -> Self {
        Gear::new_from_kind(GearKind::EStaticMeter, value.box_clone())
    }
}
