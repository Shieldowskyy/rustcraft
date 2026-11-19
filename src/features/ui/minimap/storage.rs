//! Zapis/odczyt waypointów na dysku oraz próbkowanie wysokości i cieniowania (hillshading) terenu.

use crate::features::blocks::{BlockId, Face};
use crate::features::ui::draw::pack_color;
use crate::features::world::{CHUNK_SIZE_Y, Chunk, World};

use super::types::Waypoint;

fn waypoints_file_path(slot: u8, seed: u32) -> std::path::PathBuf {
    let save_file = crate::features::world::storage::save_path(slot);
    let p = std::path::Path::new(&save_file);
    let filename = format!("slot_{}_seed_{:08X}_waypoints.txt", slot, seed);
    if let Some(parent) = p.parent() {
        parent.join(filename)
    } else {
        std::path::PathBuf::from(filename)
    }
}

pub fn serialize_waypoints(waypoints: &[Waypoint]) -> String {
    let mut out = String::new();
    for wp in waypoints {
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|{:08X}\n",
            if wp.enabled { "1" } else { "0" },
            wp.name,
            wp.x,
            wp.y,
            wp.z,
            wp.color
        ));
    }
    out
}

pub fn parse_waypoints_str(content: &str) -> Vec<Waypoint> {
    let mut waypoints = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 6 {
            let enabled = parts[0] == "1";
            let name = parts[1].to_string();
            if let (Ok(x), Ok(y), Ok(z), Ok(color)) = (
                parts[2].parse::<i32>(),
                parts[3].parse::<i32>(),
                parts[4].parse::<i32>(),
                u32::from_str_radix(parts[5].trim_start_matches("0x"), 16),
            ) {
                waypoints.push(Waypoint {
                    name,
                    x,
                    y,
                    z,
                    color: if color <= 0x00FFFFFF { 0xFF000000 | color } else { color },
                    enabled,
                });
            }
        }
    }
    waypoints
}

pub fn load_waypoints(slot: u8, seed: u32) -> Vec<Waypoint> {
    let path = waypoints_file_path(slot, seed);
    if let Ok(content) = std::fs::read_to_string(path) {
        parse_waypoints_str(&content)
    } else {
        Vec::new()
    }
}

pub fn save_waypoints(slot: u8, seed: u32, waypoints: &[Waypoint]) {
    let path = waypoints_file_path(slot, seed);
    let out = serialize_waypoints(waypoints);
    let _ = std::fs::write(path, out);
}

#[inline(always)]
pub fn sample_surface(world: &World, wx: i32, wz: i32) -> (i32, BlockId) {
    let cx = wx >> 4;
    let cz = wz >> 4;
    let lx = wx.rem_euclid(16);
    let lz = wz.rem_euclid(16);

    if let Some(chunk) = world.chunks.get(&(cx, cz)) {
        for y in (0..CHUNK_SIZE_Y).rev() {
            let blk = chunk.get_local(lx, y, lz);
            if blk != BlockId::Air {
                return (y, blk);
            }
        }
    }
    (0, BlockId::Air)
}

#[inline(always)]
pub fn sample_chunk_column(chunk: &Chunk, lx: i32, lz: i32) -> (i32, BlockId) {
    for y in (0..CHUNK_SIZE_Y).rev() {
        let blk = chunk.get_local(lx, y, lz);
        if blk != BlockId::Air {
            return (y, blk);
        }
    }
    (0, BlockId::Air)
}

#[inline(always)]
pub fn sample_chunk_column_y(chunk: &Chunk, lx: i32, lz: i32) -> i32 {
    for y in (0..CHUNK_SIZE_Y).rev() {
        let blk = chunk.get_local(lx, y, lz);
        if blk != BlockId::Air {
            return y;
        }
    }
    0
}

#[inline(always)]
pub fn fast_calculate_map_color(y: i32, blk: BlockId, y_north: i32) -> u32 {
    if blk == BlockId::Air {
        return pack_color(0.08, 0.08, 0.12);
    }

    let diff = (y - y_north).clamp(-4, 4);
    let shade = (1.0 + diff as f32 * 0.10).clamp(0.45, 1.45);

    if blk == BlockId::Water {
        let depth = ((40 - y).max(1) as f32 * 0.04).clamp(0.0, 0.45);
        let r = (0.10 - depth * 0.15).max(0.02) * shade;
        let g = (0.35 - depth * 0.25).max(0.08) * shade;
        let b = (0.75 - depth * 0.20).max(0.20) * shade;
        pack_color(r, g, b)
    } else {
        let base = blk.face_color(Face::Top);
        pack_color(
            (base[0] * shade).clamp(0.0, 1.0),
            (base[1] * shade).clamp(0.0, 1.0),
            (base[2] * shade).clamp(0.0, 1.0),
        )
    }
}
