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

/// Deployable tripwire. Beeps loudly when a ghost crosses its detection radius.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct MotionSensor {
    pub enabled: bool,
    /// Time left showing the TRIGGERED state.
    pub triggered_secs_left: f32,
    /// Cooldown before the sensor can trigger again.
    pub cooldown_secs: f32,
}

/// Detection radius in tiles.
const DETECTION_RADIUS_TILES: f32 = 4.0;

impl MotionSensor {
    /// True when any ghost is within detection range on the same floor.
    fn ghost_in_range(&self, gs: &GearStuff, pos: &Position) -> bool {
        let r2 = DETECTION_RADIUS_TILES * DETECTION_RADIUS_TILES;
        gs.bf
            .ghost_positions
            .iter()
            .any(|gp| gp.z.round() == pos.z.round() && gp.distance2(pos) < r2)
    }
}

impl GearUsable for MotionSensor {
    fn can_enable(&self) -> bool {
        true
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        // Single sprite; the triggered state is conveyed by the status text
        // and the alarm sound.
        GearSpriteID::MotionSensor
    }

    fn get_display_name(&self) -> &'static str {
        "Motion Sensor"
    }

    fn get_description(&self) -> &'static str {
        "Shoots an infrared beam that if cut will make the device beep. Can alert if a presence passes through."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.is_enabled() {
            if self.triggered_secs_left > 0.0 {
                "!TRIGGERED!".to_string()
            } else if self.enabled {
                "Armed".to_string()
            } else {
                String::new()
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
        if self.triggered_secs_left > 0.0 {
            self.triggered_secs_left -= dt;
        }
        if self.cooldown_secs > 0.0 {
            self.cooldown_secs -= dt;
        }
        if !self.enabled || self.cooldown_secs > 0.0 {
            return;
        }
        if self.ghost_in_range(gs, pos) {
            self.triggered_secs_left = 2.5;
            // Long cooldown keeps the alarm from becoming white noise.
            self.cooldown_secs = 8.0;
            let mut rng = random_seed::rng();
            let vol: f32 = rng.random_range(0.5..0.65);
            gs.play_audio("sounds/effects-chirp-click.ogg".into(), vol, pos);
            gs.play_audio("sounds/effects-dingdingding.ogg".into(), vol * 0.6, pos);
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<MotionSensor> for Gear {
    fn from(value: MotionSensor) -> Self {
        Gear::new_from_kind(GearKind::MotionSensor, value.box_clone())
    }
}
