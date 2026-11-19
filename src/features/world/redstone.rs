//! Ruststone circuit propagation (`update_ruststone_network`) and timed trigger ticks (`update_ruststone`).
//! Propagates BFS signal across wires, lamps, levers, buttons, pressure plates, and TNT.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;

use super::constants::CARDINAL_OFFSETS;
use super::world_state::World;

impl World {
    pub fn update_ruststone_network(&mut self, center: IVec3, range: i32) {
        let mut sources = Vec::new();
        let mut wires = HashSet::new();
        let mut lamps = HashSet::new();
        let mut doors = HashSet::new();
        let mut tnts = Vec::new();

        for dy in -range..=range {
            for dz in -range..=range {
                for dx in -range..=range {
                    let pos = center + IVec3::new(dx, dy, dz);
                    match self.get_block(pos) {
                        BlockId::RustStoneBlock | BlockId::LeverOn | BlockId::ButtonPressed | BlockId::PressurePlatePressed => sources.push(pos),
                        BlockId::RustStone | BlockId::RustStoneWireLit => { wires.insert(pos); }
                        BlockId::RustStoneLamp | BlockId::RustStoneLampLit => { lamps.insert(pos); }
                        BlockId::Door | BlockId::DoorOpen => { doors.insert(pos); }
                        BlockId::Tnt => tnts.push(pos),
                        _ => {}
                    }
                }
            }
        }

        let mut wire_power: HashMap<IVec3, u8> = HashMap::new();
        let mut powered_lamps = HashSet::new();
        let mut powered_doors = HashSet::new();
        let mut tnts_to_ignite = Vec::new();
        let mut queue = VecDeque::new();

        for s_pos in sources {
            for &n_off in &CARDINAL_OFFSETS {
                let n_pos = s_pos + n_off;
                if wires.contains(&n_pos) && wire_power.get(&n_pos).copied().unwrap_or(0) < 15 {
                    wire_power.insert(n_pos, 15);
                    queue.push_back((n_pos, 15));
                }
                if lamps.contains(&n_pos) { powered_lamps.insert(n_pos); }
                if doors.contains(&n_pos) { powered_doors.insert(n_pos); }
                if tnts.contains(&n_pos) { tnts_to_ignite.push(n_pos); }
            }
        }

        while let Some((w_pos, pwr)) = queue.pop_front() {
            for &n_off in &CARDINAL_OFFSETS {
                let n_pos = w_pos + n_off;
                if lamps.contains(&n_pos) { powered_lamps.insert(n_pos); }
                if doors.contains(&n_pos) { powered_doors.insert(n_pos); }
                if tnts.contains(&n_pos) { tnts_to_ignite.push(n_pos); }
                if pwr > 1 && wires.contains(&n_pos) {
                    let next_pwr = pwr - 1;
                    if wire_power.get(&n_pos).copied().unwrap_or(0) < next_pwr {
                        wire_power.insert(n_pos, next_pwr);
                        queue.push_back((n_pos, next_pwr));
                    }
                }
            }
        }

        for tnt_pos in tnts_to_ignite { self.prime_tnt(tnt_pos); }

        for w_pos in wires {
            let pwr = wire_power.get(&w_pos).copied().unwrap_or(0);
            let cur = self.get_block(w_pos);
            if pwr > 0 && cur == BlockId::RustStone { self.set_block(w_pos, BlockId::RustStoneWireLit); }
            else if pwr == 0 && cur == BlockId::RustStoneWireLit { self.set_block(w_pos, BlockId::RustStone); }
        }

        for l_pos in lamps {
            let is_pw = powered_lamps.contains(&l_pos);
            let cur = self.get_block(l_pos);
            if is_pw && cur == BlockId::RustStoneLamp { self.set_block(l_pos, BlockId::RustStoneLampLit); }
            else if !is_pw && cur == BlockId::RustStoneLampLit { self.set_block(l_pos, BlockId::RustStoneLamp); }
        }

        for d_pos in doors {
            let is_pw = powered_doors.contains(&d_pos);
            let cur = self.get_block(d_pos);
            if is_pw && cur == BlockId::Door {
                self.set_block(d_pos, BlockId::DoorOpen);
                self.ruststone_sound_events.push((d_pos, true));
            } else if !is_pw && cur == BlockId::DoorOpen {
                self.set_block(d_pos, BlockId::Door);
                self.ruststone_sound_events.push((d_pos, false));
            }
        }
    }

    pub fn update_ruststone(&mut self, dt: f32, player_pos: Vec3, mob_positions: &[Vec3]) {
        let mut expired = Vec::new();
        for (&pos, timer) in &mut self.button_timers {
            *timer -= dt;
            if *timer <= 0.0 { expired.push(pos); }
        }
        for pos in expired {
            self.button_timers.remove(&pos);
            if self.get_block(pos) == BlockId::ButtonPressed {
                self.set_block(pos, BlockId::Button);
                self.ruststone_sound_events.push((pos, false));
                self.update_ruststone_network(pos, 16);
            }
        }

        let (px, py, pz) = (player_pos.x.floor() as i32, player_pos.y.floor() as i32, player_pos.z.floor() as i32);
        let is_entity_on_plate = |pos: IVec3, ent: Vec3| -> bool {
            ent.x.floor() as i32 == pos.x && ent.z.floor() as i32 == pos.z
            && (ent.y.floor() as i32 == pos.y || (ent.y - 0.2).floor() as i32 == pos.y)
        };
        let any_on = |pos: IVec3| is_entity_on_plate(pos, player_pos) || mob_positions.iter().any(|&m| is_entity_on_plate(pos, m));

        let mut changed_plates = Vec::new();
        for &plate_pos in &self.active_pressure_plates {
            if !any_on(plate_pos) && self.get_block(plate_pos) == BlockId::PressurePlatePressed {
                changed_plates.push((plate_pos, BlockId::PressurePlate, false));
            }
        }

        for dy in -3..=3 {
            for dz in -12..=12 {
                for dx in -12..=12 {
                    let test_pos = IVec3::new(px + dx, py + dy, pz + dz);
                    if self.get_block(test_pos) == BlockId::PressurePlate && any_on(test_pos) {
                        changed_plates.push((test_pos, BlockId::PressurePlatePressed, true));
                    }
                }
            }
        }

        for (pos, new_block, is_pressed) in changed_plates {
            self.set_block(pos, new_block);
            self.ruststone_sound_events.push((pos, is_pressed));
            self.update_ruststone_network(pos, 16);
        }
    }

}
