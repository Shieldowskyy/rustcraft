//! RustCraft Data-Driven Modding Engine
//!
//! Provides a zero-dependency, cross-platform mod loader supporting:
//! - Custom voxel blocks with physical properties, collision, and custom textures
//! - Custom items, tools, and food with nutrition and hunger restoration
//! - Custom crafting recipes (1x1, 2x2, 3x3 grids)
//! - Furnace smelting recipes
//! - Custom block breaking drops and mob death loot tables
//! - In-game custom chat commands with action execution (/give, /heal, /fly, /message, etc.)
//! - Procedural and pixel-art texture synthesis for mod assets without external image files
//!
//! Mods are loaded dynamically from the `./mods/` directory at startup.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use crate::gameplay::chat::{ChatState, CommandContext};
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::recipe::{Recipe, RecipeBook};
use crate::graphics::atlas::{Texture, TextureId, TEX_SIZE};
use crate::world::block::{BlockId, Face};

// ============================================================================
// Mod Data Definitions
// ============================================================================

/// Metadata describing an installed mod.
#[derive(Clone, Debug, PartialEq)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

/// Pattern styles for procedural texture generation in mods.
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

/// Definition of a custom block or item registered by a mod.
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

/// Actions executable by custom mod chat commands.
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

/// Definition of a custom chat command introduced by a mod.
#[derive(Clone, Debug, PartialEq)]
pub struct ModCommandDef {
    pub name: String,
    pub description: String,
    pub actions: Vec<ModAction>,
}

/// Mob loot drop definition from a mod.
#[derive(Clone, Debug, PartialEq)]
pub struct ModMobDrop {
    pub mod_id: String,
    pub mob_name: String,
    pub item: BlockId,
    pub count: u8,
    pub chance: f32,
}

// ============================================================================
// Global Mod Registry
// ============================================================================

struct ModRegistry {
    manifests: Vec<ModManifest>,
    blocks_by_id: HashMap<u8, CustomBlockDef>,
    id_by_identifier: HashMap<String, BlockId>,
    smelting_recipes: HashMap<BlockId, BlockId>,
    mob_drops: Vec<ModMobDrop>,
    commands: HashMap<String, ModCommandDef>,
    cached_creative_blocks: Vec<BlockId>,
    cached_creative_items: Vec<BlockId>,
    raw_mod_files: Vec<crate::net::protocol::ModSyncEntry>,
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

fn get_registry() -> &'static RwLock<ModRegistry> {
    REGISTRY.get_or_init(|| RwLock::new(ModRegistry::new()))
}

// ============================================================================
// Public Query API (Zero-allocation, thread-safe)
// ============================================================================

/// Returns whether the given custom block ID has solid collision.
pub fn is_custom_solid(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.solid && !b.is_item_only;
        }
    }
    true
}

/// Returns whether the custom item is item-only (cannot be placed in the world as a block).
pub fn is_custom_item_only(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.is_item_only;
        }
    }
    false
}

/// Returns whether the given custom block ID is fully opaque.
pub fn is_custom_opaque(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.opaque && !b.is_item_only;
        }
    }
    true
}

/// Returns whether the given custom block ID is transparent (e.g. glass-like).
pub fn is_custom_transparent(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.transparent;
        }
    }
    false
}

/// Returns the localized/display name for a custom block or item.
pub fn get_custom_name(id: u8) -> &'static str {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.name;
        }
    }
    "Modded Item"
}

/// Returns the base RGB face color for a custom block.
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
    [0.7, 0.2, 0.8] // Default vibrant purple fallback
}

/// Returns the TextureId assigned to this custom block or item.
pub fn get_custom_texture_id(id: u8) -> TextureId {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.texture_id;
        }
    }
    TextureId::Custom(0)
}

/// Returns whether a custom block or item is edible food.
pub fn is_custom_food(id: u8) -> bool {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.is_food;
        }
    }
    false
}

/// Returns (hunger_restored, saturation) if the custom item is food.
pub fn get_custom_nutrition(id: u8) -> Option<(f32, f32)> {
    if let Ok(reg) = get_registry().read() {
        if let Some(b) = reg.blocks_by_id.get(&id) {
            return b.nutrition;
        }
    }
    None
}

/// Returns the item dropped when a custom or vanilla block is mined.
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

/// Returns all custom drops for a defeated mob (pig, cow, chicken, zombie, skeleton, reaper).
pub fn get_custom_mob_drops(mob_name: &str) -> Vec<(BlockId, u8)> {
    let mut drops = Vec::new();
    let lower = mob_name.to_ascii_lowercase();
    if let Ok(reg) = get_registry().read() {
        for drop in &reg.mob_drops {
            if drop.mob_name == lower {
                // Probabilistic check
                let r = (rand_hash(drop.item.to_u8() as u32, drops.len() as u32) % 1000) as f32 / 1000.0;
                if r <= drop.chance {
                    drops.push((drop.item, drop.count.max(1)));
                }
            }
        }
    }
    drops
}

/// Returns the smelting output for an input item if a mod recipe exists.
pub fn get_custom_smelting(input: BlockId) -> Option<BlockId> {
    if let Ok(reg) = get_registry().read() {
        return reg.smelting_recipes.get(&input).copied();
    }
    None
}

/// Resolves a string identifier (e.g. "emerald", "ruby_ore") to a `BlockId`.
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

/// Looks up a custom mod chat command.
pub fn find_mod_command(cmd: &str) -> Option<ModCommandDef> {
    let lower = cmd.trim().to_ascii_lowercase();
    if let Ok(reg) = get_registry().read() {
        return reg.commands.get(&lower).cloned();
    }
    None
}

/// Returns a list of all loaded mod commands `(name, description)`.
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

/// Returns all custom mod block definitions for the creative block inventory tab.
pub fn get_custom_creative_blocks() -> Vec<BlockId> {
    if let Ok(reg) = get_registry().read() {
        return reg.cached_creative_blocks.clone();
    }
    Vec::new()
}

/// Returns all custom mod item definitions for the creative items inventory tab.
pub fn get_custom_creative_items() -> Vec<BlockId> {
    if let Ok(reg) = get_registry().read() {
        return reg.cached_creative_items.clone();
    }
    Vec::new()
}

/// Generates a vector of custom textures registered by mods for `TextureAtlas`.
pub fn generate_mod_textures() -> Vec<Texture> {
    let mut textures = Vec::new();
    if let Ok(reg) = get_registry().read() {
        let mut sorted: Vec<&CustomBlockDef> = reg.blocks_by_id.values().collect();
        sorted.sort_by_key(|b| b.id);
        for block in sorted {
            textures.push(block.texture.clone());
        }
    }
    textures
}

/// Executes a custom mod chat command with its declared actions.
pub fn execute_mod_command(cmd_def: &ModCommandDef, chat: &mut ChatState, ctx: &mut CommandContext) {
    for action in &cmd_def.actions {
        match action {
            ModAction::Message(msg) => {
                chat.add_info(msg);
            }
            ModAction::Give { item, count } => {
                let resolved = if let Some(vanilla) = crate::gameplay::chat::parse_block_id(item) {
                    Some(vanilla)
                } else {
                    find_custom_block_by_name(item)
                };

                if let Some(block) = resolved {
                    let mut remaining = *count;
                    // Try hotbar first
                    for slot in &mut ctx.player.inventory.hotbar {
                        if remaining == 0 {
                            break;
                        }
                        if let Some(st) = slot {
                            if st.block == block && st.count < 64 {
                                let add = remaining.min(64 - st.count);
                                st.count += add;
                                remaining -= add;
                            }
                        } else {
                            let add = remaining.min(64);
                            *slot = Some(ItemStack::new(block, add));
                            remaining -= add;
                        }
                    }
                    // Try main inventory
                    for slot in &mut ctx.player.inventory.main {
                        if remaining == 0 {
                            break;
                        }
                        if let Some(st) = slot {
                            if st.block == block && st.count < 64 {
                                let add = remaining.min(64 - st.count);
                                st.count += add;
                                remaining -= add;
                            }
                        } else {
                            let add = remaining.min(64);
                            *slot = Some(ItemStack::new(block, add));
                            remaining -= add;
                        }
                    }
                    chat.add_success(format!("Gave {} {} to player", count, block.name()));
                } else {
                    chat.add_error(format!("Unknown item in command: {}", item));
                }
            }
            ModAction::Heal => {
                ctx.player.health = 20.0;
                ctx.player.hunger = 20.0;
                ctx.player.saturation = 5.0;
                ctx.player.exhaustion = 0.0;
                chat.add_success("Player restored to full health and hunger.");
            }
            ModAction::FlyToggle => {
                *ctx.is_flying = !*ctx.is_flying;
                chat.add_info(format!("Flight mode: {}", if *ctx.is_flying { "ENABLED" } else { "DISABLED" }));
            }
            ModAction::Time(ticks) => {
                let frac = (ticks % 24000.0) / 24000.0;
                ctx.world.time_of_day = crate::world::World::DAY_DURATION * frac;
                chat.add_success(format!("Time set to {:.0} ticks", ticks));
            }
            ModAction::SpawnMob { mob, count } => {
                let kind_opt = match mob.to_ascii_lowercase().as_str() {
                    "pig" => Some(crate::gameplay::mob::MobKind::Pig),
                    "cow" => Some(crate::gameplay::mob::MobKind::Cow),
                    "chicken" => Some(crate::gameplay::mob::MobKind::Chicken),
                    "zombie" => Some(crate::gameplay::mob::MobKind::Zombie),
                    "skeleton" => Some(crate::gameplay::mob::MobKind::Skeleton),
                    "reaper" => Some(crate::gameplay::mob::MobKind::Reaper),
                    "spider" => Some(crate::gameplay::mob::MobKind::Spider),
                    _ => None,
                };
                if let Some(kind) = kind_opt {
                    let player_pos = ctx.camera.position;
                    for _ in 0..*count {
                        ctx.mob_manager.spawn_mob(kind, player_pos);
                    }
                    chat.add_success(format!("Spawned {} {}", count, kind.name()));
                } else {
                    chat.add_error(format!("Unknown mob type: {}", mob));
                }
            }
            ModAction::ClearInventory => {
                ctx.player.inventory.hotbar = [None; 10];
                ctx.player.inventory.main = [None; 27];
                ctx.player.inventory.held_item = None;
                chat.add_success("Player inventory cleared.");
            }
        }
    }
}

// ============================================================================
// Mod Loader Implementation
// ============================================================================

/// Primary mod manager managing lifecycle and scanning `mods/`.
pub struct ModManager;

impl ModManager {
    /// Returns the active `mods/` directory path on disk.
    pub fn mods_dir() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            let base = crate::world::storage::app_files_dir();
            let dir = base.join("mods");
            if dir.is_dir() {
                return dir;
            }
        }
        #[cfg(target_os = "vita")]
        {
            let custom_dir = PathBuf::from("ux0:data/rustcraft/mods");
            if custom_dir.is_dir() {
                return custom_dir;
            }
            let app_dir = PathBuf::from("app0:mods");
            if app_dir.is_dir() {
                return app_dir;
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let dir = parent.join("mods");
                if dir.is_dir() {
                    return dir;
                }
            }
        }
        PathBuf::from("mods")
    }

    /// Loads or registers a mod directly from its JSON content string (e.g. received over multiplayer network).
    pub fn register_mod_from_str(
        json_content: &str,
        mod_dir: &Path,
        recipe_book: &mut RecipeBook,
    ) -> Result<ModManifest, String> {
        Self::parse_and_register_mod(json_content, mod_dir, recipe_book)
    }

    /// Returns all active mod JSON payloads for network synchronization with multiplayer clients.
    pub fn get_active_mod_files() -> Vec<crate::net::protocol::ModSyncEntry> {
        if let Ok(reg) = get_registry().read() {
            return reg.raw_mod_files.clone();
        }
        Vec::new()
    }

    /// Initializes mod loader and parses all manifests found in `mods/`.
    pub fn load_mods(recipe_book: &mut RecipeBook) {
        let mods_path = Self::mods_dir();
        if !mods_path.exists() {
            let _ = fs::create_dir_all(&mods_path);
            crate::log_info!("MODS", "Created empty mods directory at '{}'", mods_path.display());
            return;
        }

        let entries = match fs::read_dir(&mods_path) {
            Ok(e) => e,
            Err(err) => {
                crate::log_warn!("MODS", "Could not read mods directory: {}", err);
                return;
            }
        };

        crate::log_info!("MODS", "Scanning '{}' for mods...", mods_path.display());
        let mut loaded_count = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_file = path.join("mod.json");
                if manifest_file.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_file) {
                        match Self::parse_and_register_mod(&content, &path, recipe_book) {
                            Ok(manifest) => {
                                crate::log_info!(
                                    "MODS",
                                    "Successfully loaded mod '{}' [{}] v{} by {}",
                                    manifest.name,
                                    manifest.id,
                                    manifest.version,
                                    manifest.author
                                );
                                loaded_count += 1;
                            }
                            Err(err) => {
                                crate::log_error!(
                                    "MODS",
                                    "Failed to load mod from '{}': {}",
                                    path.display(),
                                    err
                                );
                            }
                        }
                    }
                }
            }
        }

        crate::log_info!("MODS", "Mod loading complete. {} mod(s) active.", loaded_count);
    }

    fn parse_and_register_mod(
        json_content: &str,
        _mod_dir: &Path,
        recipe_book: &mut RecipeBook,
    ) -> Result<ModManifest, String> {
        let root = parse_json_value_root(json_content)?;
        let obj = root.as_object().ok_or("mod.json must contain a root object")?;

        let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("unknown_mod").to_string();
        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
        let version = obj.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string();
        let author = obj.get("author").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let description = obj.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let manifest = ModManifest {
            id: id.clone(),
            name,
            version,
            author,
            description,
        };

        let mut reg = get_registry().write().map_err(|e| e.to_string())?;

        // 1. Parse Blocks & Items
        if let Some(blocks_array) = obj.get("blocks").and_then(|v| v.as_array()) {
            for block_val in blocks_array {
                if let Some(block_obj) = block_val.as_object() {
                    Self::register_block_or_item(block_obj, false, &mut reg)?;
                }
            }
        }

        if let Some(items_array) = obj.get("items").and_then(|v| v.as_array()) {
            for item_val in items_array {
                if let Some(item_obj) = item_val.as_object() {
                    Self::register_block_or_item(item_obj, true, &mut reg)?;
                }
            }
        }

        // Post-process drops so blocks can reference items or blocks declared in any order
        let drop_updates: Vec<(u8, BlockId)> = reg
            .blocks_by_id
            .iter()
            .filter_map(|(&id, def)| {
                let d_name = def.drop_item_name.as_ref()?;
                let clean = d_name.trim().to_ascii_lowercase().replace('-', "_");
                let resolved = reg.id_by_identifier.get(&clean).copied().or_else(|| crate::gameplay::chat::parse_vanilla_block_id(&clean))?;
                Some((id, resolved))
            })
            .collect();

        for (id, drop_id) in drop_updates {
            if let Some(def) = reg.blocks_by_id.get_mut(&id) {
                def.drop_block = Some(drop_id);
            }
        }

        // 2. Parse Recipes
        if let Some(recipes_array) = obj.get("recipes").and_then(|v| v.as_array()) {
            for rec_val in recipes_array {
                if let Some(rec_obj) = rec_val.as_object() {
                    if let Some(recipe) = Self::parse_recipe(rec_obj, &reg) {
                        recipe_book.add(recipe);
                    }
                }
            }
        }

        // 3. Parse Smelting Recipes
        if let Some(smelt_array) = obj.get("smelting").and_then(|v| v.as_array()) {
            for s_val in smelt_array {
                if let Some(s_obj) = s_val.as_object() {
                    let in_name = s_obj.get("input").and_then(|v| v.as_str()).unwrap_or("");
                    let out_name = s_obj.get("output").and_then(|v| v.as_str()).unwrap_or("");
                    let in_id = Self::resolve_identifier(in_name, &reg);
                    let out_id = Self::resolve_identifier(out_name, &reg);
                    if let (Some(i), Some(o)) = (in_id, out_id) {
                        reg.smelting_recipes.insert(i, o);
                    }
                }
            }
        }

        // 4. Parse Mob Loot
        if let Some(loot_array) = obj.get("loot").and_then(|v| v.as_array()) {
            reg.mob_drops.retain(|d| d.mod_id != id);
            for l_val in loot_array {
                if let Some(l_obj) = l_val.as_object() {
                    let mob = l_obj.get("target").and_then(|v| v.as_str()).unwrap_or("").to_ascii_lowercase();
                    let item_name = l_obj.get("item").and_then(|v| v.as_str()).unwrap_or("");
                    let count = l_obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
                    let chance = l_obj.get("chance").and_then(|v| v.as_f32()).unwrap_or(1.0);
                    if let Some(item_id) = Self::resolve_identifier(item_name, &reg) {
                        reg.mob_drops.push(ModMobDrop {
                            mod_id: id.clone(),
                            mob_name: mob,
                            item: item_id,
                            count,
                            chance,
                        });
                    }
                }
            }
        }

        // 5. Parse Chat Commands
        if let Some(cmds_array) = obj.get("commands").and_then(|v| v.as_array()) {
            for cmd_val in cmds_array {
                if let Some(cmd_obj) = cmd_val.as_object() {
                    let cmd_name = cmd_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_ascii_lowercase();
                    let desc = cmd_obj.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let mut actions = Vec::new();

                    if let Some(act_array) = cmd_obj.get("actions").and_then(|v| v.as_array()) {
                        for act_val in act_array {
                            if let Some(act_obj) = act_val.as_object() {
                                let act_type = act_obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                match act_type {
                                    "message" | "echo" => {
                                        let text = act_obj.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        actions.push(ModAction::Message(text));
                                    }
                                    "give" => {
                                        let item = act_obj.get("item").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let count = act_obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
                                        actions.push(ModAction::Give { item, count });
                                    }
                                    "heal" => {
                                        actions.push(ModAction::Heal);
                                    }
                                    "fly" => {
                                        actions.push(ModAction::FlyToggle);
                                    }
                                    "time" => {
                                        let ticks = act_obj.get("ticks").and_then(|v| v.as_f32()).unwrap_or(1000.0);
                                        actions.push(ModAction::Time(ticks));
                                    }
                                    "spawn" => {
                                        let mob = act_obj.get("mob").and_then(|v| v.as_str()).unwrap_or("pig").to_string();
                                        let count = act_obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
                                        actions.push(ModAction::SpawnMob { mob, count });
                                    }
                                    "clear" => {
                                        actions.push(ModAction::ClearInventory);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if !cmd_name.is_empty() {
                        reg.commands.insert(cmd_name.clone(), ModCommandDef {
                            name: cmd_name,
                            description: desc,
                            actions,
                        });
                    }
                }
            }
        }

        if let Some(pos) = reg.manifests.iter().position(|m| m.id == manifest.id) {
            reg.manifests[pos] = manifest.clone();
        } else {
            reg.manifests.push(manifest.clone());
        }

        if let Some(pos) = reg.raw_mod_files.iter().position(|m| m.mod_id == manifest.id) {
            reg.raw_mod_files[pos] = crate::net::protocol::ModSyncEntry {
                mod_id: manifest.id.clone(),
                content: json_content.to_string(),
            };
        } else {
            reg.raw_mod_files.push(crate::net::protocol::ModSyncEntry {
                mod_id: manifest.id.clone(),
                content: json_content.to_string(),
            });
        }

        Ok(manifest)
    }

    fn register_block_or_item(
        obj: &HashMap<String, JsonNode>,
        is_item_only: bool,
        reg: &mut ModRegistry,
    ) -> Result<(), String> {
        // ID range for custom blocks/items: 140..=254
        let id_raw = obj.get("id").and_then(|v| v.as_u8()).ok_or("block/item requires 'id' (140-255)")?;
        if id_raw < 140 {
            return Err(format!("Mod block/item ID {} conflicts with vanilla IDs (0-139). Use 140..255.", id_raw));
        }

        let identifier = obj.get("identifier").and_then(|v| v.as_str()).ok_or("requires 'identifier'")?.to_string();
        let display_name_str = obj.get("name").and_then(|v| v.as_str()).unwrap_or(&identifier);
        let name: &'static str = Box::leak(display_name_str.to_string().into_boxed_str());

        let solid = obj.get("solid").and_then(|v| v.as_bool()).unwrap_or(!is_item_only);
        let opaque = obj.get("opaque").and_then(|v| v.as_bool()).unwrap_or(!is_item_only);
        let transparent = obj.get("transparent").and_then(|v| v.as_bool()).unwrap_or(false);
        let hardness = obj.get("hardness").and_then(|v| v.as_f32()).unwrap_or(1.5);
        let light = obj.get("light").and_then(|v| v.as_u8()).unwrap_or(0);

        let is_food = obj.get("is_food").and_then(|v| v.as_bool()).unwrap_or(false);
        let nutrition = if is_food {
            if let Some(arr) = obj.get("nutrition").and_then(|v| v.as_array()) {
                let h = arr.first().and_then(|v| v.as_f32()).unwrap_or(4.0);
                let s = arr.get(1).and_then(|v| v.as_f32()).unwrap_or(2.0);
                Some((h, s))
            } else {
                Some((4.0, 2.4))
            }
        } else {
            None
        };

        // Texture generation
        let pattern = if let Some(tex_obj) = obj.get("texture").and_then(|v| v.as_object()) {
            Self::parse_texture_pattern(tex_obj)
        } else {
            TexturePattern::MineralBlock {
                base_color: [0.2, 0.7, 0.4],
                border_color: [0.1, 0.4, 0.2],
            }
        };

        let texture = Self::synthesize_texture(&pattern);
        let avg_color = Self::compute_texture_average_color(&texture);
        let face_colors = [
            avg_color,
            [avg_color[0] * 0.7, avg_color[1] * 0.7, avg_color[2] * 0.7],
            [avg_color[0] * 0.85, avg_color[1] * 0.85, avg_color[2] * 0.85],
            [avg_color[0] * 0.85, avg_color[1] * 0.85, avg_color[2] * 0.85],
            [avg_color[0] * 0.8, avg_color[1] * 0.8, avg_color[2] * 0.8],
            [avg_color[0] * 0.8, avg_color[1] * 0.8, avg_color[2] * 0.8],
        ];

        let custom_tex_idx = (id_raw - 140) as u8;
        let texture_id = TextureId::Custom(custom_tex_idx);

        // Optional drop definition
        let (drop_item_name, drop_count) = if let Some(drop_obj) = obj.get("drops").and_then(|v| v.as_object()) {
            let drop_name = drop_obj.get("item").and_then(|v| v.as_str()).unwrap_or(&identifier).to_string();
            let drop_c = drop_obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
            (Some(drop_name), drop_c)
        } else {
            (None, 1)
        };

        let initial_drop = if let Some(ref d_name) = drop_item_name {
            Self::resolve_identifier(d_name, reg).unwrap_or(BlockId::Custom(id_raw))
        } else {
            BlockId::Custom(id_raw)
        };

        let def = CustomBlockDef {
            id: id_raw,
            identifier: identifier.clone(),
            name,
            is_item_only,
            solid,
            opaque,
            transparent,
            hardness,
            light_emission: light,
            is_food,
            nutrition,
            face_colors,
            texture_id,
            texture,
            drop_item_name,
            drop_block: Some(initial_drop),
            drop_count,
        };

        let block_id = BlockId::Custom(id_raw);
        reg.blocks_by_id.insert(id_raw, def);
        reg.id_by_identifier.insert(identifier.to_ascii_lowercase(), block_id);

        if is_item_only {
            if !reg.cached_creative_items.contains(&block_id) {
                reg.cached_creative_items.push(block_id);
            }
        } else {
            if !reg.cached_creative_blocks.contains(&block_id) {
                reg.cached_creative_blocks.push(block_id);
            }
        }

        Ok(())
    }

    fn parse_texture_pattern(obj: &HashMap<String, JsonNode>) -> TexturePattern {
        let pattern_type = obj.get("pattern").and_then(|v| v.as_str()).unwrap_or("mineral");
        match pattern_type {
            "ore" => {
                let bg = obj.get("background_color").and_then(|v| v.as_color3()).unwrap_or([0.55, 0.55, 0.55]);
                let ore = obj.get("ore_color").and_then(|v| v.as_color3()).unwrap_or([0.1, 0.9, 0.4]);
                TexturePattern::Ore {
                    background_color: bg,
                    ore_color: ore,
                }
            }
            "speckled" => {
                let base = obj.get("base_color").and_then(|v| v.as_color3()).unwrap_or([0.3, 0.8, 0.3]);
                let var = obj.get("variance").and_then(|v| v.as_f32()).unwrap_or(0.15);
                TexturePattern::Speckled {
                    base_color: base,
                    variance: var,
                }
            }
            "gem" | "crystal" => {
                let col = obj.get("color").and_then(|v| v.as_color3()).unwrap_or([0.1, 0.95, 0.45]);
                TexturePattern::Gem { color: col }
            }
            "apple" | "food" => {
                let col = obj.get("color").and_then(|v| v.as_color3()).unwrap_or([0.9, 0.1, 0.2]);
                TexturePattern::Apple { color: col }
            }
            "ingot" => {
                let col = obj.get("color").and_then(|v| v.as_color3()).unwrap_or([0.8, 0.85, 0.9]);
                TexturePattern::Ingot { color: col }
            }
            "sword" => {
                let blade = obj.get("blade_color").and_then(|v| v.as_color3()).unwrap_or([0.1, 0.9, 0.4]);
                let handle = obj.get("handle_color").and_then(|v| v.as_color3()).unwrap_or([0.45, 0.28, 0.14]);
                TexturePattern::ToolSword { blade_color: blade, handle_color: handle }
            }
            "pickaxe" => {
                let head = obj.get("head_color").and_then(|v| v.as_color3()).unwrap_or([0.1, 0.9, 0.4]);
                let handle = obj.get("handle_color").and_then(|v| v.as_color3()).unwrap_or([0.45, 0.28, 0.14]);
                TexturePattern::ToolPickaxe { head_color: head, handle_color: handle }
            }
            _ => {
                let base = obj.get("base_color").and_then(|v| v.as_color3()).unwrap_or([0.2, 0.7, 0.4]);
                let border = obj.get("border_color").and_then(|v| v.as_color3()).unwrap_or([
                    base[0] * 0.6,
                    base[1] * 0.6,
                    base[2] * 0.6,
                ]);
                TexturePattern::MineralBlock {
                    base_color: base,
                    border_color: border,
                }
            }
        }
    }

    fn synthesize_texture(pattern: &TexturePattern) -> Texture {
        let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

        match pattern {
            TexturePattern::Speckled { base_color, variance } => {
                for y in 0..TEX_SIZE {
                    for x in 0..TEX_SIZE {
                        let n = (rand_hash(x as u32, y as u32) % 1000) as f32 / 1000.0 - 0.5;
                        let shade = 1.0 + n * variance;
                        texels[y * TEX_SIZE + x] = [
                            (base_color[0] * shade).clamp(0.0, 1.0),
                            (base_color[1] * shade).clamp(0.0, 1.0),
                            (base_color[2] * shade).clamp(0.0, 1.0),
                        ];
                    }
                }
            }
            TexturePattern::Ore { background_color, ore_color } => {
                for y in 0..TEX_SIZE {
                    for x in 0..TEX_SIZE {
                        let stone_n = (rand_hash(x as u32, y as u32) % 1000) as f32 / 1000.0 - 0.5;
                        let stone_shade = 1.0 + stone_n * 0.15;
                        texels[y * TEX_SIZE + x] = [
                            (background_color[0] * stone_shade).clamp(0.0, 1.0),
                            (background_color[1] * stone_shade).clamp(0.0, 1.0),
                            (background_color[2] * stone_shade).clamp(0.0, 1.0),
                        ];
                    }
                }
                // Flecks of ore clusters
                let ore_pixels = [
                    (3, 4), (4, 4), (4, 5), (3, 5),
                    (9, 2), (10, 2), (10, 3), (11, 3),
                    (6, 8), (7, 8), (7, 9), (8, 9), (6, 9),
                    (11, 10), (12, 10), (12, 11),
                    (2, 12), (3, 12), (3, 13),
                ];
                for (ox, oy) in ore_pixels {
                    if ox < TEX_SIZE && oy < TEX_SIZE {
                        let sparkle = if (ox + oy) % 2 == 0 { 1.15 } else { 0.95 };
                        texels[oy * TEX_SIZE + ox] = [
                            (ore_color[0] * sparkle).clamp(0.0, 1.0),
                            (ore_color[1] * sparkle).clamp(0.0, 1.0),
                            (ore_color[2] * sparkle).clamp(0.0, 1.0),
                        ];
                    }
                }
            }
            TexturePattern::MineralBlock { base_color, border_color } => {
                for y in 0..TEX_SIZE {
                    for x in 0..TEX_SIZE {
                        let is_border = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1
                            || x == 1 || x == TEX_SIZE - 2 || y == 1 || y == TEX_SIZE - 2;
                        let n = (rand_hash(x as u32, y as u32) % 1000) as f32 / 1000.0 - 0.5;
                        let col = if is_border {
                            *border_color
                        } else {
                            let shade = 1.0 + n * 0.12;
                            [
                                (base_color[0] * shade).clamp(0.0, 1.0),
                                (base_color[1] * shade).clamp(0.0, 1.0),
                                (base_color[2] * shade).clamp(0.0, 1.0),
                            ]
                        };
                        texels[y * TEX_SIZE + x] = col;
                    }
                }
            }
            TexturePattern::Gem { color } => {
                let dark = [color[0] * 0.5, color[1] * 0.5, color[2] * 0.5];
                let light = [(color[0] * 1.3).min(1.0), (color[1] * 1.3).min(1.0), (color[2] * 1.3).min(1.0)];
                for y in 0..TEX_SIZE {
                    for x in 0..TEX_SIZE {
                        // Diamond-shaped gem icon
                        let dx = (x as i32 - 8).abs();
                        let dy = (y as i32 - 8).abs();
                        if dx + dy <= 6 {
                            let col = if x < 8 && y < 8 {
                                light
                            } else if x >= 8 && y >= 8 {
                                dark
                            } else {
                                *color
                            };
                            texels[y * TEX_SIZE + x] = col;
                        }
                    }
                }
            }
            TexturePattern::Apple { color } => {
                let dark = [color[0] * 0.6, color[1] * 0.6, color[2] * 0.6];
                let stem = [0.45, 0.28, 0.14];
                let leaf = [0.2, 0.8, 0.2];
                for y in 0..TEX_SIZE {
                    for x in 0..TEX_SIZE {
                        let dx = (x as i32 - 8).abs();
                        let dy = (y as i32 - 9).abs();
                        if dx * dx + dy * dy <= 20 {
                            let col = if x > 8 { dark } else { *color };
                            texels[y * TEX_SIZE + x] = col;
                        } else if x == 8 && (y == 3 || y == 4) {
                            texels[y * TEX_SIZE + x] = stem;
                        } else if x == 9 && y == 3 {
                            texels[y * TEX_SIZE + x] = leaf;
                        }
                    }
                }
            }
            TexturePattern::Ingot { color } => {
                let dark = [color[0] * 0.65, color[1] * 0.65, color[2] * 0.65];
                let light = [(color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0)];
                for y in 5..12 {
                    for x in 3..14 {
                        let col = if y == 5 || x == 3 {
                            light
                        } else if y == 11 || x == 13 {
                            dark
                        } else {
                            *color
                        };
                        texels[y * TEX_SIZE + x] = col;
                    }
                }
            }
            TexturePattern::ToolSword { blade_color, handle_color } => {
                for i in 2..10 {
                    texels[i * TEX_SIZE + (15 - i)] = *blade_color;
                    texels[i * TEX_SIZE + (14 - i)] = [blade_color[0] * 0.8, blade_color[1] * 0.8, blade_color[2] * 0.8];
                }
                texels[10 * TEX_SIZE + 4] = [0.2, 0.2, 0.2];
                texels[10 * TEX_SIZE + 5] = [0.2, 0.2, 0.2];
                texels[10 * TEX_SIZE + 6] = [0.2, 0.2, 0.2];
                texels[11 * TEX_SIZE + 4] = *handle_color;
                texels[12 * TEX_SIZE + 3] = *handle_color;
                texels[13 * TEX_SIZE + 2] = [handle_color[0] * 0.6, handle_color[1] * 0.6, handle_color[2] * 0.6];
            }
            TexturePattern::ToolPickaxe { head_color, handle_color } => {
                for i in 2..10 {
                    texels[(14 - i) * TEX_SIZE + (14 - i)] = *handle_color;
                }
                let head_coords = [(11, 2), (12, 2), (13, 3), (13, 4), (4, 11), (3, 12), (2, 12), (2, 13)];
                for (hx, hy) in head_coords {
                    texels[hy * TEX_SIZE + hx] = *head_color;
                }
                texels[3 * TEX_SIZE + 12] = [head_color[0] * 1.2, head_color[1] * 1.2, head_color[2] * 1.2];
            }
            TexturePattern::Pixels(pxs) => {
                for (i, p) in pxs.iter().enumerate().take(TEX_SIZE * TEX_SIZE) {
                    texels[i] = *p;
                }
            }
        }

        Texture { texels }
    }

    fn compute_texture_average_color(tex: &Texture) -> [f32; 3] {
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        let mut count = 0.0;
        for c in &tex.texels {
            if c[0] > 0.001 || c[1] > 0.001 || c[2] > 0.001 {
                r += c[0];
                g += c[1];
                b += c[2];
                count += 1.0;
            }
        }
        if count == 0.0 {
            return [0.5, 0.5, 0.5];
        }
        [r / count, g / count, b / count]
    }

    fn parse_recipe(obj: &HashMap<String, JsonNode>, reg: &ModRegistry) -> Option<Recipe> {
        let grid_val = obj.get("grid").and_then(|v| v.as_array())?;
        let result_name = obj.get("result").and_then(|v| v.as_str())?;
        let count = obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
        let result_block = Self::resolve_identifier(result_name, reg)?;

        let height = grid_val.len();
        if height == 0 || height > 3 {
            return None;
        }

        let first_row = grid_val.first()?.as_array()?;
        let width = first_row.len();
        if width == 0 || width > 3 {
            return None;
        }

        let mut pattern = Vec::with_capacity(width * height);
        for row_node in grid_val {
            let row = row_node.as_array()?;
            if row.len() != width {
                return None;
            }
            for item_node in row {
                let s = item_node.as_str().unwrap_or("").trim();
                if s.is_empty() || s == "_" || s == "air" {
                    pattern.push(None);
                } else {
                    let id = Self::resolve_identifier(s, reg)?;
                    pattern.push(Some(id));
                }
            }
        }

        Some(Recipe {
            width,
            height,
            pattern,
            result: ItemStack::new(result_block, count),
        })
    }

    fn resolve_identifier(id_str: &str, reg: &ModRegistry) -> Option<BlockId> {
        let clean = id_str.trim().to_ascii_lowercase().replace('-', "_");
        if clean.is_empty() || clean == "_" || clean == "air" {
            return Some(BlockId::Air);
        }
        if let Some(id) = reg.id_by_identifier.get(&clean) {
            return Some(*id);
        }
        crate::gameplay::chat::parse_vanilla_block_id(&clean)
    }
}

// ============================================================================
// Deterministic Hash Helper
// ============================================================================

fn rand_hash(x: u32, y: u32) -> u32 {
    let mut n = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}

// ============================================================================
// Zero-Dependency JSON Parser for Mod Data
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JsonNode {
    Str(String),
    Number(f64),
    Bool(bool),
    Null,
    Array(Vec<JsonNode>),
    Object(HashMap<String, JsonNode>),
}

impl JsonNode {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            JsonNode::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_f32(&self) -> Option<f32> {
        match self {
            JsonNode::Number(n) => Some(*n as f32),
            _ => None,
        }
    }

    pub(crate) fn as_u8(&self) -> Option<u8> {
        match self {
            JsonNode::Number(n) => Some(*n as u8),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            JsonNode::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[JsonNode]> {
        match self {
            JsonNode::Array(a) => Some(a),
            _ => None,
        }
    }

    pub(crate) fn as_object(&self) -> Option<&HashMap<String, JsonNode>> {
        match self {
            JsonNode::Object(o) => Some(o),
            _ => None,
        }
    }

    pub(crate) fn as_color3(&self) -> Option<[f32; 3]> {
        match self {
            JsonNode::Array(arr) if arr.len() >= 3 => {
                let r = arr[0].as_f32().unwrap_or(0.0);
                let g = arr[1].as_f32().unwrap_or(0.0);
                let b = arr[2].as_f32().unwrap_or(0.0);
                Some([r, g, b])
            }
            JsonNode::Str(hex) if hex.starts_with('#') && hex.len() == 7 => {
                let r = u8::from_str_radix(&hex[1..3], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[3..5], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[5..7], 16).ok()? as f32 / 255.0;
                Some([r, g, b])
            }
            _ => None,
        }
    }
}

pub(crate) fn parse_json_value_root(input: &str) -> Result<JsonNode, String> {
    // Strip // single-line comments for user friendliness
    let mut cleaned = String::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        cleaned.push_str(line);
        cleaned.push('\n');
    }

    let chars: Vec<char> = cleaned.chars().collect();
    let mut idx = 0;
    skip_whitespace(&chars, &mut idx);
    let (node, _) = parse_node(&chars, idx)?;
    Ok(node)
}

fn skip_whitespace(chars: &[char], idx: &mut usize) {
    while *idx < chars.len() && chars[*idx].is_whitespace() {
        *idx += 1;
    }
}

fn parse_node(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    skip_whitespace(chars, &mut idx);
    if idx >= chars.len() {
        return Err("Unexpected end of JSON input".to_string());
    }

    match chars[idx] {
        '{' => parse_object(chars, idx),
        '[' => parse_array(chars, idx),
        '"' => {
            let (s, next) = parse_string(chars, idx)?;
            Ok((JsonNode::Str(s), next))
        }
        't' | 'f' => parse_bool(chars, idx),
        'n' => parse_null(chars, idx),
        '-' | '0'..='9' => parse_number(chars, idx),
        other => Err(format!("Unexpected character '{}' at offset {}", other, idx)),
    }
}

fn parse_object(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    idx += 1; // skip '{'
    let mut map = HashMap::new();

    loop {
        skip_whitespace(chars, &mut idx);
        if idx >= chars.len() {
            return Err("Unclosed object".to_string());
        }
        if chars[idx] == '}' {
            idx += 1;
            break;
        }

        if chars[idx] != '"' {
            return Err(format!("Expected string key in object at offset {}", idx));
        }
        let (key, next_after_key) = parse_string(chars, idx)?;
        idx = next_after_key;

        skip_whitespace(chars, &mut idx);
        if idx >= chars.len() || chars[idx] != ':' {
            return Err(format!("Expected ':' after key '{}' at offset {}", key, idx));
        }
        idx += 1; // skip ':'

        let (val, next_after_val) = parse_node(chars, idx)?;
        idx = next_after_val;
        map.insert(key, val);

        skip_whitespace(chars, &mut idx);
        if idx < chars.len() && chars[idx] == ',' {
            idx += 1;
        } else if idx < chars.len() && chars[idx] == '}' {
            idx += 1;
            break;
        }
    }

    Ok((JsonNode::Object(map), idx))
}

fn parse_array(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    idx += 1; // skip '['
    let mut arr = Vec::new();

    loop {
        skip_whitespace(chars, &mut idx);
        if idx >= chars.len() {
            return Err("Unclosed array".to_string());
        }
        if chars[idx] == ']' {
            idx += 1;
            break;
        }

        let (val, next_after_val) = parse_node(chars, idx)?;
        idx = next_after_val;
        arr.push(val);

        skip_whitespace(chars, &mut idx);
        if idx < chars.len() && chars[idx] == ',' {
            idx += 1;
        } else if idx < chars.len() && chars[idx] == ']' {
            idx += 1;
            break;
        }
    }

    Ok((JsonNode::Array(arr), idx))
}

fn parse_string(chars: &[char], mut idx: usize) -> Result<(String, usize), String> {
    idx += 1; // skip leading '"'
    let mut s = String::new();
    while idx < chars.len() {
        let c = chars[idx];
        if c == '"' {
            idx += 1;
            return Ok((s, idx));
        } else if c == '\\' {
            idx += 1;
            if idx >= chars.len() {
                return Err("Dangling escape in string".to_string());
            }
            match chars[idx] {
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                '/' => s.push('/'),
                'b' => s.push('\x08'),
                'f' => s.push('\x0c'),
                'n' => s.push('\n'),
                'r' => s.push('\r'),
                't' => s.push('\t'),
                _ => s.push(chars[idx]),
            }
        } else {
            s.push(c);
        }
        idx += 1;
    }
    Err("Unclosed string literal".to_string())
}

fn parse_bool(chars: &[char], idx: usize) -> Result<(JsonNode, usize), String> {
    let rem: String = chars[idx..].iter().take(5).collect();
    if rem.starts_with("true") {
        Ok((JsonNode::Bool(true), idx + 4))
    } else if rem.starts_with("false") {
        Ok((JsonNode::Bool(false), idx + 5))
    } else {
        Err(format!("Invalid boolean token at offset {}", idx))
    }
}

fn parse_null(chars: &[char], idx: usize) -> Result<(JsonNode, usize), String> {
    let rem: String = chars[idx..].iter().take(4).collect();
    if rem.starts_with("null") {
        Ok((JsonNode::Null, idx + 4))
    } else {
        Err(format!("Invalid token at offset {}", idx))
    }
}

fn parse_number(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    let start = idx;
    if chars[idx] == '-' {
        idx += 1;
    }
    while idx < chars.len() && (chars[idx].is_ascii_digit() || chars[idx] == '.' || chars[idx] == 'e' || chars[idx] == 'E' || chars[idx] == '+' || chars[idx] == '-') {
        idx += 1;
    }
    let s: String = chars[start..idx].iter().collect();
    let num = s.parse::<f64>().map_err(|e| format!("Bad number '{}': {}", s, e))?;
    Ok((JsonNode::Number(num), idx))
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parser_basic() {
        let json = r#"
        // This is a comment
        {
            "id": "emerald_mod",
            "name": "Emerald Mod",
            "version": "1.0",
            "blocks": [
                {
                    "id": 85,
                    "identifier": "emerald_ore",
                    "solid": true,
                    "hardness": 3.0
                }
            ],
            "commands": [
                {
                    "name": "emerald_kit",
                    "actions": [
                        { "type": "give", "item": "emerald_ore", "count": 16 }
                    ]
                }
            ]
        }
        "#;
        let root = parse_json_value_root(json).expect("JSON should parse");
        let obj = root.as_object().unwrap();
        assert_eq!(obj.get("id").unwrap().as_str().unwrap(), "emerald_mod");
        let blocks = obj.get("blocks").unwrap().as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        let ore = blocks[0].as_object().unwrap();
        assert_eq!(ore.get("id").unwrap().as_u8().unwrap(), 85);
        assert_eq!(ore.get("hardness").unwrap().as_f32().unwrap(), 3.0);
    }

    #[test]
    fn test_mod_registration_and_queries() {
        let json = r#"{
            "id": "test_mod",
            "name": "Test Mod",
            "blocks": [
                {
                    "id": 212,
                    "identifier": "test_ruby_ore",
                    "name": "Ruby Ore",
                    "solid": true,
                    "hardness": 4.5,
                    "texture": {
                        "pattern": "ore",
                        "ore_color": [1.0, 0.0, 0.2]
                    }
                }
            ],
            "items": [
                {
                    "id": 213,
                    "identifier": "test_ruby",
                    "name": "Ruby Gem",
                    "is_food": false
                }
            ],
            "smelting": [
                { "input": "test_ruby_ore", "output": "test_ruby" }
            ]
        }"#;

        let mut book = RecipeBook::new();
        let manifest = ModManager::parse_and_register_mod(json, Path::new("mods/test"), &mut book).unwrap();
        assert_eq!(manifest.id, "test_mod");

        assert_eq!(find_custom_block_by_name("test_ruby_ore"), Some(BlockId::Custom(212)));
        assert_eq!(find_custom_block_by_name("test_ruby"), Some(BlockId::Custom(213)));
        assert_eq!(get_custom_name(212), "Ruby Ore");
        assert_eq!(get_custom_name(213), "Ruby Gem");
        assert!(is_custom_solid(212));
        assert_eq!(get_custom_smelting(BlockId::Custom(212)), Some(BlockId::Custom(213)));
    }

    #[test]
    fn test_multiplayer_mod_sync() {
        let json = r#"{
            "id": "net_mod",
            "name": "Network Sync Mod",
            "blocks": [
                {
                    "id": 214,
                    "identifier": "net_block",
                    "name": "Network Block"
                }
            ]
        }"#;

        let mut book = RecipeBook::new();
        let manifest = ModManager::register_mod_from_str(json, Path::new("mods/net_mod"), &mut book).unwrap();
        assert_eq!(manifest.id, "net_mod");

        let active_files = ModManager::get_active_mod_files();
        assert!(active_files.iter().any(|m| m.mod_id == "net_mod" && m.content.contains("Network Block")));
        assert_eq!(find_custom_block_by_name("net_block"), Some(BlockId::Custom(214)));
    }

    #[test]
    fn test_mod_food_and_nutrition() {
        let json = r#"{
            "id": "food_mod",
            "name": "Food Mod",
            "items": [
                {
                    "id": 149,
                    "identifier": "super_pie",
                    "name": "Super Pie",
                    "is_food": true,
                    "nutrition": [8.0, 5.0]
                }
            ]
        }"#;

        let mut book = RecipeBook::new();
        let _ = ModManager::register_mod_from_str(json, Path::new("mods/food"), &mut book).unwrap();

        assert!(is_custom_food(149));
        assert_eq!(get_custom_nutrition(149), Some((8.0, 5.0)));
    }

    #[test]
    fn test_mod_crafting_recipes_match_grid() {
        let json = r#"{
            "id": "craft_mod",
            "name": "Craft Mod",
            "blocks": [
                { "id": 220, "identifier": "craft_block", "name": "Craft Block" }
            ],
            "items": [
                { "id": 221, "identifier": "craft_gem", "name": "Craft Gem" }
            ],
            "recipes": [
                {
                    "type": "shaped",
                    "grid": [
                        ["craft_gem", "craft_gem"],
                        ["craft_gem", "craft_gem"]
                    ],
                    "result": "craft_block",
                    "count": 1
                }
            ]
        }"#;

        let mut book = RecipeBook::new();
        let _ = ModManager::register_mod_from_str(json, Path::new("mods/craft"), &mut book).unwrap();

        let gem_item = Some(crate::gameplay::inventory::ItemStack::new(BlockId::Custom(221), 1));

        let grid_2x2 = [gem_item, gem_item, gem_item, gem_item];
        let result = book.match_grid(&grid_2x2, 2, 2);
        assert!(result.is_some());
        let res_stack = result.unwrap();
        assert_eq!(res_stack.block, BlockId::Custom(220));
        assert_eq!(res_stack.count, 1);
    }

    #[test]
    fn test_mod_block_custom_drops_and_mob_loot() {
        let json = r#"{
            "id": "loot_mod",
            "name": "Loot Mod",
            "blocks": [
                {
                    "id": 225,
                    "identifier": "crystal_ore",
                    "name": "Crystal Ore",
                    "drops": { "item": "crystal_shard", "count": 2 }
                }
            ],
            "items": [
                { "id": 226, "identifier": "crystal_shard", "name": "Crystal Shard" }
            ],
            "loot": [
                { "target": "zombie", "item": "crystal_shard", "count": 3, "chance": 1.0 }
            ]
        }"#;

        let mut book = RecipeBook::new();
        let _ = ModManager::register_mod_from_str(json, Path::new("mods/loot"), &mut book).unwrap();

        // Custom block break drop
        let drop = get_custom_block_drop(BlockId::Custom(225));
        assert_eq!(drop, Some((BlockId::Custom(226), 2)));

        // Mob loot
        let mob_drops = get_custom_mob_drops("zombie");
        assert!(mob_drops.iter().any(|&(b, count)| b == BlockId::Custom(226) && count == 3));
    }

    #[test]
    fn test_mod_command_dispatch() {
        let json = r#"{
            "id": "cmd_mod",
            "name": "Cmd Mod",
            "commands": [
                {
                    "name": "super_heal",
                    "description": "Heals player to max",
                    "actions": [
                        { "type": "heal" },
                        { "type": "message", "text": "Super healed!" }
                    ]
                }
            ]
        }"#;

        let mut book = RecipeBook::new();
        let _ = ModManager::register_mod_from_str(json, Path::new("mods/cmd"), &mut book).unwrap();

        let mut chat = crate::gameplay::chat::ChatState::new();
        let mut mob_manager = crate::gameplay::mob::MobManager::new();
        let mut player = crate::gameplay::player::Player::new();
        player.health = 5.0;
        player.hunger = 2.0;

        let mut camera = crate::graphics::Camera::new(crate::core::math::Vec3::new(0.0, 64.0, 0.0));
        let mut world = crate::world::World::new();
        let mut items = Vec::new();
        let mut particles = crate::gameplay::particles::ParticleSystem::new();
        let mut is_flying = false;
        let mut fly_vel = crate::core::math::Vec3::zero();
        let mut horiz_vel = crate::core::math::Vec3::zero();
        let mut reload_chunks = false;

        let mut ctx = crate::gameplay::chat::CommandContext {
            player: &mut player,
            camera: &mut camera,
            world: &mut world,
            mob_manager: &mut mob_manager,
            items: &mut items,
            particles: &mut particles,
            is_flying: &mut is_flying,
            fly_velocity: &mut fly_vel,
            horiz_velocity: &mut horiz_vel,
            reload_chunks_requested: &mut reload_chunks,
        };

        chat.execute_command("/super_heal", &mut ctx);
        assert_eq!(player.health, player.max_health);
        assert_eq!(player.hunger, player.max_hunger);
        assert!(chat.messages.iter().any(|m| m.text.contains("Super healed!")));
    }

    #[test]
    fn test_mod_bundled_emerald_expansion_file_validity() {
        let path = Path::new("mods/emerald_expansion/mod.json");
        assert!(path.exists(), "Reference mod file mods/emerald_expansion/mod.json must exist");
        let content = std::fs::read_to_string(path).expect("Must read emerald_expansion mod.json");

        let mut book = RecipeBook::new();
        let manifest = ModManager::register_mod_from_str(&content, path.parent().unwrap(), &mut book)
            .expect("Reference mod must register cleanly without errors");

        assert_eq!(manifest.id, "emerald_expansion");
        assert!(find_custom_block_by_name("emerald_ore").is_some());
        assert!(find_custom_block_by_name("emerald").is_some());
        assert!(find_custom_block_by_name("emerald_apple").is_some());
        assert!(find_custom_block_by_name("ruby_ore").is_some());
        assert!(find_custom_block_by_name("ruby").is_some());
        assert!(find_custom_block_by_name("ruby_block").is_some());
    }
}
