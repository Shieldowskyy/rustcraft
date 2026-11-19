//! Typy danych systemu modów (`ModManifest`, `CustomBlockDef`, `TexturePattern`, `ModAction`, `ModMobDrop`) oraz stałe zakresu ID.

use crate::features::blocks::BlockId;
use crate::features::rendering::atlas::{Texture, TextureId};

pub(super) const MOD_BLOCK_ID_MIN: u8 = 186;
pub(super) const MOD_BLOCK_ID_MAX: u8 = 249;

#[derive(Clone, Debug, PartialEq)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TexturePattern {
    Speckled {
        base_color: [f32; 3],
        variance: f32,
    },
    Ore {
        background_color: [f32; 3],
        ore_color: [f32; 3],
    },
    MineralBlock {
        base_color: [f32; 3],
        border_color: [f32; 3],
    },
    Gem {
        color: [f32; 3],
    },
    Apple {
        color: [f32; 3],
    },
    Ingot {
        color: [f32; 3],
    },
    ToolSword {
        blade_color: [f32; 3],
        handle_color: [f32; 3],
    },
    ToolPickaxe {
        head_color: [f32; 3],
        handle_color: [f32; 3],
    },
    Pixels(Vec<[f32; 3]>),
}

#[derive(Clone, Debug)]
pub struct CustomBlockDef {
    pub id: u8,
    pub identifier: String,
    pub name: &'static str,
    pub is_item_only: bool,
    pub solid: bool,
    pub opaque: bool,
    pub transparent: bool,
    pub hardness: f32,
    pub light_emission: u8,
    pub is_food: bool,
    pub nutrition: Option<(f32, f32)>,
    pub face_colors: [[f32; 3]; 6],
    pub texture_id: TextureId,
    pub texture: Texture,
    pub drop_item_name: Option<String>,
    pub drop_block: Option<BlockId>,
    pub drop_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModAction {
    Message(String),
    Give { item: String, count: u8 },
    Heal,
    FlyToggle,
    Time(f32),
    SpawnMob { mob: String, count: u8 },
    ClearInventory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModCommandDef {
    pub name: String,
    pub description: String,
    pub actions: Vec<ModAction>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModMobDrop {
    pub mod_id: String,
    pub mob_name: String,
    pub item: BlockId,
    pub count: u8,
    pub chance: f32,
}

pub(super) fn rand_hash(x: u32, y: u32) -> u32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
