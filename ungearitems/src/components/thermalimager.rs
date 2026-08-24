use bevy::prelude::*;
use rand::Rng as _;
use uncore::components::board::boardposition::BoardPosition;
use uncore::kelvin_to_celsius;
use uncore::random_seed;
use uncore::{
    components::board::position::Position,
    types::gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID},
};
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;

use super::{Gear, GearKind, on_off};

/// Heat-vision scanner. Sweeps the surrounding tiles and reports the coldest
/// spot it can see, making ghost rooms easy to find at a glance.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct ThermalImager {
    pub enabled: bool,
    /// Coldest temperature found in the last sweep (Kelvin).
    pub spot_temp_k: f32,
    /// Direction label of the cold spot ("N", "SW", ...).
    pub spot_dir: String,
    pub refresh_timer: f32,
}

fn dir_label(dx: i64, dy: i64) -> &'static str {
    match (dx, dy) {
        (0, 0) => "here",
        (-1, -1) => "NW",
        (0, -1) => "N",
        (1, -1) => "NE",
        (-1, 0) => "W",
        (1, 0) => "E",
        (-1, 1) => "SW",
        (0, 1) => "S",
        (1, 1) => "SE",
        _ => "?",
    }
}

impl ThermalImager {
    /// Sweeps the temperature field around the tool and returns the coldest
    /// tile found as `(kelvin, direction label)`.
    fn sweep(&self, gs: &GearStuff, pos: &Position) -> (f32, String) {
        let bpos = pos.to_board_position();
        let mut coldest_k = gs
            .bf
            .temperature_field
            .get(bpos.ndidx())
            .copied()
            .unwrap_or(gs.bf.ambient_temp);
        let mut coldest_dx: i64 = 0;
        let mut coldest_dy: i64 = 0;

        const RADIUS: i64 = 5;
        for dy in -RADIUS..=RADIUS {
            for dx in -RADIUS..=RADIUS {
                let p = BoardPosition {
                    x: bpos.x + dx,
                    y: bpos.y + dy,
                    z: bpos.z,
                };
                let Some(t) = gs.bf.temperature_field.get(p.ndidx()).copied() else {
                    continue;
                };
                if t < coldest_k {
                    coldest_k = t;
                    coldest_dx = dx;
                    coldest_dy = dy;
                }
            }
        }
        // Snap the offset to the dominant axis pair for a coarse 8-way label.
        let (lx, ly) = (coldest_dx.signum(), coldest_dy.signum());
        (coldest_k, dir_label(lx, ly).to_string())
    }
}

impl GearUsable for ThermalImager {
    fn can_enable(&self) -> bool {
        true
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.enabled {
            GearSpriteID::ThermalImagerOn
        } else {
            GearSpriteID::ThermalImagerOff
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Thermal Imager"
    }

    fn get_description(&self) -> &'static str {
        "Heat vision to see easily what's hot and what's cold. Might improve visibility of the paranormal and haunted objects."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.is_enabled() {
            let spot_c = kelvin_to_celsius(self.spot_temp_k);
            if spot_c < 3.0 {
                format!("Ambient scan\nSpot: {:.1}°C {} ❄", spot_c, self.spot_dir)
            } else {
                format!(
                    "Ambient scan\nWarmest area: {:.1}°C {}",
                    spot_c, self.spot_dir
                )
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
        if !self.enabled {
            return;
        }
        let dt = gs.time.delta_secs();
        self.refresh_timer -= dt;
        if self.refresh_timer <= 0.0 {
            self.refresh_timer = 0.4;
            let (temp_k, dir) = self.sweep(gs, pos);
            self.spot_temp_k = temp_k;
            self.spot_dir = dir;
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
            self.refresh_timer = 0.6;
            self.spot_dir = "??".to_string();
        }
    }
}

impl From<ThermalImager> for Gear {
    fn from(value: ThermalImager) -> Self {
        Gear::new_from_kind(GearKind::ThermalImager, value.box_clone())
    }
}
