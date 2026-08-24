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

/// Detects ionized particle trails left behind by paranormal activity. The
/// reading rises near the breach and along the paths ghosts travel.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct IonMeter {
    pub enabled: bool,
    /// Smoothed displayed reading (eV).
    pub display: f32,
    /// Fast-responding raw sensor value (eV).
    pub raw: f32,
}

impl IonMeter {
    /// Signal contribution in eV from miasma and nearby ghost activity.
    fn measure(&self, gs: &GearStuff, pos: &Position) -> f32 {
        let mut rng = random_seed::rng();
        let bpos = pos.to_board_position();

        // Ambient ionization from the miasma field around the tool.
        let mut signal = 0.0;
        for p in bpos.iter_xy_neighbors_nosize(2) {
            let pressure = gs
                .bf
                .miasma
                .pressure_field
                .get(p.ndidx())
                .copied()
                .unwrap_or(0.0);
            signal += pressure * 60.0;
        }

        // Charged trail left by nearby ghosts (same floor only).
        for gp in &gs.bf.ghost_positions {
            if gp.z.round() != pos.z.round() {
                continue;
            }
            let dist2 = gp.distance2(pos);
            signal += 9000.0 / (dist2 + 30.0);
        }

        // Sensor noise floor.
        let noise: f32 = rng.random_range(-8.0..8.0);
        signal + 15.0 + noise
    }
}

impl GearUsable for IonMeter {
    fn can_enable(&self) -> bool {
        true
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        if !self.enabled {
            return GearSpriteID::IonMeterOff;
        }
        if self.display >= 250.0 {
            GearSpriteID::IonMeter2
        } else if self.display >= 90.0 {
            GearSpriteID::IonMeter1
        } else {
            GearSpriteID::IonMeter0
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Ion Meter"
    }

    fn get_description(&self) -> &'static str {
        "Detects charged particles in the air. Ghost leave a trace as they move and this tool may help following the ghost."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.is_enabled() {
            let reading = self.display.max(0.0);
            if reading >= 250.0 {
                format!("Ionization: {:.0} eV\nTrace detected", reading)
            } else {
                format!("Ionization: {:.0} eV", reading)
            }
        } else {
            String::new()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        self.enabled = !self.enabled;
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let dt = gs.time.delta_secs();
        if !self.enabled {
            // Readings fade out when switched off.
            self.raw *= 1.0 - (dt * 4.0).min(1.0);
            self.display += (self.raw - self.display) * (dt * 3.0).min(1.0);
            return;
        }

        self.raw = self.measure(gs, pos);
        // Display lags behind the raw sensor to feel analog.
        self.display =
            (self.display + (self.raw - self.display) * (dt * 5.0).min(1.0)).clamp(-50.0, 2000.0);

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
            // Spikes scramble the display for a moment.
            self.display += rng.random_range(100.0..400.0);
        }
    }
}

impl From<IonMeter> for Gear {
    fn from(value: IonMeter) -> Self {
        Gear::new_from_kind(GearKind::IonMeter, value.box_clone())
    }
}
