//! Globalny wątkowo-bezpieczny rejestr załadowanych modów (`ModRegistry`) oraz publiczne API zapytań o bloki, loot, przepisy i tekstury.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::features::blocks::{BlockId, Face};
use crate::features::net::protocol::ModSyncEntry;
use crate::features::rendering::atlas::{Texture, TextureId, TEX_SIZE};

use super::types::{
    rand_hash, CustomBlockDef, ModCommandDef, ModManifest, ModMobDrop, MOD_BLOCK_ID_MIN,
};

pub(super) struct ModRegistry {
    pub(super) manifests: Vec<ModManifest>,
    pub(super) blocks_by_id: HashMap<u8, CustomBlockDef>,
    pub(super) id_by_identifier: HashMap<String, BlockId>,
    pub(super) smelting_recipes: HashMap<BlockId, BlockId>,
    pub(super) mob_drops: Vec<ModMobDrop>,
    pub(super) commands: HashMap<String, ModCommandDef>,
    pub(super) cached_creative_blocks: Vec<BlockId>,
    pub(super) cached_creative_items: Vec<BlockId>,
    pub(super) raw_mod_files: Vec<ModSyncEntry>,
}

impl ModRegistry {
    fn new() -> Self {
        Self {
            manifests: Vec::new(),
            blocks_by_id: HashMap::new(),
            id_by_identifier: HashMap::new(),
            smelting_recipes: HashMap::new(),
            mob_drops: Vec::new(),
            commands: HashMap::new(),
            cached_creative_blocks: Vec::new(),
            cached_creative_items: Vec::new(),
            raw_mod_files: Vec::new(),
        }
    }
}

static REGISTRY: OnceLock<RwLock<ModRegistry>> = OnceLock::new();

pub(super) fn get_registry() -> &'static RwLock<ModRegistry> {
    REGISTRY.get_or_init(|| RwLock::new(ModRegistry::new()))
}

pub fn is_custom_solid(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.solid && !b.is_item_only;
        }
    }
    true
}

pub fn is_custom_item_only(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.is_item_only;
        }
    }
    false
}

pub fn is_custom_opaque(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.opaque && !b.is_item_only;
        }
    }
    true
}

pub fn is_custom_transparent(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.transparent;
        }
    }
    false
}

pub fn get_custom_name(id: u8) -> &'static str {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.name;
        }
    }
    "Modded Item"
}

pub fn get_custom_face_color(id: u8, face: Face) -> [f32; 3] {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            let idx = match face {
                Face::Top => 0,
                Face::Bottom => 1,
                Face::North => 2,
                Face::South => 3,
                Face::East => 4,
                Face::West => 5,
            };
            return b.face_colors[idx];
        }
    }
    [0.7, 0.2, 0.8]
}

pub fn get_custom_texture_id(id: u8) -> TextureId {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.texture_id;
        }
    }
    TextureId::Custom(0)
}

pub fn is_custom_food(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.is_food;
        }
    }
    false
}

pub fn get_custom_nutrition(id: u8) -> Option<(f32, f32)> {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.nutrition;
        }
    }
    None
}

pub fn get_custom_block_drop(broken_block: BlockId) -> Option<(BlockId, u8)> {
    if let BlockId::Custom(id) = broken_block {
        if let Ok(reg) = get_registry().read() {
            if let Some(b) = reg.blocks_by_id.get(&id) {
                if let Some(drop_id) = b.drop_block {
                    return Some((drop_id, b.drop_count.max(1)));
                }
                return Some((broken_block, 1));
            }
        }
    }
    None
}

pub fn get_custom_mob_drops(mob_name: &str) -> Vec<(BlockId, u8)> {
    let mut drops = Vec::new();
    let lower = mob_name.to_ascii_lowercase();
    if let Ok(reg) = get_registry().read() {
        for drop in &reg.mob_drops {
            if drop.mob_name == lower {
                let r = (rand_hash(drop.item.to_u8() as u32, drops.len() as u32) % 1000) as f32
                    / 1000.0;
                if r <= drop.chance {
                    drops.push((drop.item, drop.count.max(1)));
                }
            }
        }
    }
    drops
}

pub fn get_custom_smelting(input: BlockId) -> Option<BlockId> {
    if let Ok(reg) = get_registry().read() {
        return reg.smelting_recipes.get(&input).copied();
    }
    None
}

pub fn find_custom_block_by_name(name: &str) -> Option<BlockId> {
    let lower = name.trim().to_ascii_lowercase().replace('-', "_");
    if let Ok(reg) = get_registry().read() {
        if let Some(id) = reg.id_by_identifier.get(&lower) {
            return Some(*id);
        }
        for (id, block) in &reg.blocks_by_id {
            if block.name.to_ascii_lowercase() == lower {
                return Some(BlockId::Custom(*id));
            }
        }
    }
    None
}

pub fn find_mod_command(cmd: &str) -> Option<ModCommandDef> {
    let lower = cmd.trim().to_ascii_lowercase();
    if let Ok(reg) = get_registry().read() {
        return reg.commands.get(&lower).cloned();
    }
    None
}

pub fn get_all_mod_commands() -> Vec<(String, String)> {
    let mut list = Vec::new();
    if let Ok(reg) = get_registry().read() {
        for (name, def) in &reg.commands {
            list.push((name.clone(), def.description.clone()));
        }
    }
    list.sort_by(|a, b| a.0.cmp(&b.0));
    list
}

pub fn get_custom_creative_blocks() -> Vec<BlockId> {
    if let Ok(reg) = get_registry().read() {
        return reg.cached_creative_blocks.clone();
    }
    Vec::new()
}

pub fn get_custom_creative_items() -> Vec<BlockId> {
    if let Ok(reg) = get_registry().read() {
        return reg.cached_creative_items.clone();
    }
    Vec::new()
}

pub fn generate_mod_textures() -> Vec<Texture> {
    let mut textures = Vec::new();
    if let Ok(reg) = get_registry().read() {
        let mut sorted: Vec<&CustomBlockDef> = reg.blocks_by_id.values().collect();
        sorted.sort_by_key(|b| b.id);
        for block in sorted {
            let texture_index = (block.id - MOD_BLOCK_ID_MIN) as usize;
            textures.resize_with(texture_index + 1, || Texture {
                texels: vec![[0.0; 3]; TEX_SIZE * TEX_SIZE],
            });
            textures[texture_index] = block.texture.clone();
        }
    }
    textures
}
