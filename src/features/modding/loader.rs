//! Ładowanie manifestów modów (`mod.json`) z dysku i sieci oraz rejestracja bloków, przepisów, lootów i komend (`ModManager`).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::features::blocks::BlockId;
use crate::features::net::protocol::ModSyncEntry;
use crate::features::player::{ItemStack, Recipe, RecipeBook};
use crate::features::rendering::atlas::TextureId;
use crate::features::ui::chat::parse_vanilla_block_id;

use super::json::{parse_json_value_root, JsonNode};
use super::registry::{get_registry, ModRegistry};
use super::textures::{compute_texture_average_color, parse_texture_pattern, synthesize_texture};
use super::types::{
    CustomBlockDef, ModAction, ModCommandDef, ModManifest, ModMobDrop, TexturePattern,
    MOD_BLOCK_ID_MAX, MOD_BLOCK_ID_MIN,
};

pub struct ModManager;

impl ModManager {
    pub fn mods_dir() -> PathBuf {
        crate::platform::storage::mods_dir()
    }

    pub fn register_mod_from_str(
        json_content: &str,
        mod_dir: &Path,
        recipe_book: &mut RecipeBook,
    ) -> Result<ModManifest, String> {
        Self::parse_and_register_mod(json_content, mod_dir, recipe_book)
    }

    pub fn get_active_mod_files() -> Vec<ModSyncEntry> {
        if let Ok(reg) = get_registry().read() {
            return reg.raw_mod_files.clone();
        }
        Vec::new()
    }

    pub fn load_mods(recipe_book: &mut RecipeBook) {
        let mods_path = Self::mods_dir();
        if !mods_path.exists() {
            let _ = fs::create_dir_all(&mods_path);
            crate::log_info!(
                "MODS",
                "Created empty mods directory at '{}'",
                mods_path.display()
            );
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

        crate::log_info!(
            "MODS",
            "Mod loading complete. {} mod(s) active.",
            loaded_count
        );
    }

    pub(super) fn parse_and_register_mod(
        json_content: &str,
        _mod_dir: &Path,
        recipe_book: &mut RecipeBook,
    ) -> Result<ModManifest, String> {
        let root = parse_json_value_root(json_content)?;
        let obj = root
            .as_object()
            .ok_or("mod.json must contain a root object")?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_mod")
            .to_string();
        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        let version = obj
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();
        let author = obj
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let manifest = ModManifest {
            id: id.clone(),
            name,
            version,
            author,
            description,
        };

        let mut reg = get_registry().write().map_err(|e| e.to_string())?;

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

        let drop_updates: Vec<(u8, BlockId)> = reg
            .blocks_by_id
            .iter()
            .filter_map(|(&id, def)| {
                let d_name = def.drop_item_name.as_ref()?;
                let clean = d_name.trim().to_ascii_lowercase().replace('-', "_");
                let resolved = reg
                    .id_by_identifier
                    .get(&clean)
                    .copied()
                    .or_else(|| parse_vanilla_block_id(&clean))?;
                Some((id, resolved))
            })
            .collect();

        for (id, drop_id) in drop_updates {
            if let Some(def) = reg.blocks_by_id.get_mut(&id) {
                def.drop_block = Some(drop_id);
            }
        }

        if let Some(recipes_array) = obj.get("recipes").and_then(|v| v.as_array()) {
            for rec_val in recipes_array {
                if let Some(rec_obj) = rec_val.as_object() {
                    if let Some(recipe) = Self::parse_recipe(rec_obj, &reg) {
                        recipe_book.add(recipe);
                    }
                }
            }
        }

        if let Some(smelt_array) = obj.get("smelting").and_then(|v| v.as_array()) {
            for s_val in smelt_array {
                if let Some(s_obj) = s_val.as_object() {
                    let in_name = s_obj.get("input").and_then(|v| v.as_str()).unwrap_or("");
                    let out_name = s_obj.get("output").and_then(|v| v.as_str()).unwrap_or("");
                    if let (Some(i), Some(o)) = (
                        Self::resolve_identifier(in_name, &reg),
                        Self::resolve_identifier(out_name, &reg),
                    ) {
                        reg.smelting_recipes.insert(i, o);
                    }
                }
            }
        }

        if let Some(loot_array) = obj.get("loot").and_then(|v| v.as_array()) {
            reg.mob_drops.retain(|d| d.mod_id != id);
            for l_val in loot_array {
                if let Some(l_obj) = l_val.as_object() {
                    let mob = l_obj
                        .get("target")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
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

        if let Some(cmds_array) = obj.get("commands").and_then(|v| v.as_array()) {
            for cmd_val in cmds_array {
                if let Some(cmd_obj) = cmd_val.as_object() {
                    Self::parse_and_insert_command(cmd_obj, &mut reg);
                }
            }
        }

        if let Some(pos) = reg.manifests.iter().position(|m| m.id == manifest.id) {
            reg.manifests[pos] = manifest.clone();
        } else {
            reg.manifests.push(manifest.clone());
        }

        let entry = ModSyncEntry {
            mod_id: manifest.id.clone(),
            content: json_content.to_string(),
        };
        if let Some(pos) = reg
            .raw_mod_files
            .iter()
            .position(|m| m.mod_id == manifest.id)
        {
            reg.raw_mod_files[pos] = entry;
        } else {
            reg.raw_mod_files.push(entry);
        }

        Ok(manifest)
    }

    fn parse_and_insert_command(cmd_obj: &HashMap<String, JsonNode>, reg: &mut ModRegistry) {
        let cmd_name = cmd_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let desc = cmd_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut actions = Vec::new();

        if let Some(act_array) = cmd_obj.get("actions").and_then(|v| v.as_array()) {
            for act_val in act_array {
                if let Some(act_obj) = act_val.as_object() {
                    let act_type = act_obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    match act_type {
                        "message" | "echo" => {
                            let text = act_obj
                                .get("text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            actions.push(ModAction::Message(text));
                        }
                        "give" => {
                            let item = act_obj
                                .get("item")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let count = act_obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
                            actions.push(ModAction::Give { item, count });
                        }
                        "heal" => actions.push(ModAction::Heal),
                        "fly" => actions.push(ModAction::FlyToggle),
                        "time" => {
                            let ticks = act_obj
                                .get("ticks")
                                .and_then(|v| v.as_f32())
                                .unwrap_or(1000.0);
                            actions.push(ModAction::Time(ticks));
                        }
                        "spawn" => {
                            let mob = act_obj
                                .get("mob")
                                .and_then(|v| v.as_str())
                                .unwrap_or("pig")
                                .to_string();
                            let count = act_obj.get("count").and_then(|v| v.as_u8()).unwrap_or(1);
                            actions.push(ModAction::SpawnMob { mob, count });
                        }
                        "clear" => actions.push(ModAction::ClearInventory),
                        _ => {}
                    }
                }
            }
        }

        if !cmd_name.is_empty() {
            reg.commands.insert(
                cmd_name.clone(),
                ModCommandDef {
                    name: cmd_name,
                    description: desc,
                    actions,
                },
            );
        }
    }

    fn register_block_or_item(
        obj: &HashMap<String, JsonNode>,
        is_item_only: bool,
        reg: &mut ModRegistry,
    ) -> Result<(), String> {
        let id_raw = obj
            .get("id")
            .and_then(|v| v.as_u8())
            .ok_or("block/item requires 'id' (186-249)")?;
        if !(MOD_BLOCK_ID_MIN..=MOD_BLOCK_ID_MAX).contains(&id_raw) {
            return Err(format!(
                "Mod block/item ID {} conflicts with vanilla IDs or exceeds the custom texture layer range. Use {}..={}.",
                id_raw, MOD_BLOCK_ID_MIN, MOD_BLOCK_ID_MAX
            ));
        }

        let identifier = obj
            .get("identifier")
            .and_then(|v| v.as_str())
            .ok_or("requires 'identifier'")?
            .to_string();
        let display_name_str = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&identifier);
        let name: &'static str = Box::leak(display_name_str.to_string().into_boxed_str());

        let solid = obj
            .get("solid")
            .and_then(|v| v.as_bool())
            .unwrap_or(!is_item_only);
        let opaque = obj
            .get("opaque")
            .and_then(|v| v.as_bool())
            .unwrap_or(!is_item_only);
        let transparent = obj
            .get("transparent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let hardness = obj.get("hardness").and_then(|v| v.as_f32()).unwrap_or(1.5);
        let light = obj.get("light").and_then(|v| v.as_u8()).unwrap_or(0);

        let is_food = obj
            .get("is_food")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
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

        let pattern = if let Some(tex_obj) = obj.get("texture").and_then(|v| v.as_object()) {
            parse_texture_pattern(tex_obj)
        } else {
            TexturePattern::MineralBlock {
                base_color: [0.2, 0.7, 0.4],
                border_color: [0.1, 0.4, 0.2],
            }
        };

        let texture = synthesize_texture(&pattern);
        let avg_color = compute_texture_average_color(&texture);
        let face_colors = [
            avg_color,
            [avg_color[0] * 0.7, avg_color[1] * 0.7, avg_color[2] * 0.7],
            [avg_color[0] * 0.85, avg_color[1] * 0.85, avg_color[2] * 0.85],
            [avg_color[0] * 0.85, avg_color[1] * 0.85, avg_color[2] * 0.85],
            [avg_color[0] * 0.8, avg_color[1] * 0.8, avg_color[2] * 0.8],
            [avg_color[0] * 0.8, avg_color[1] * 0.8, avg_color[2] * 0.8],
        ];

        let texture_id = TextureId::Custom(id_raw - MOD_BLOCK_ID_MIN);

        let (drop_item_name, drop_count) =
            if let Some(drop_obj) = obj.get("drops").and_then(|v| v.as_object()) {
                let drop_name = drop_obj
                    .get("item")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&identifier)
                    .to_string();
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
        reg.id_by_identifier
            .insert(identifier.to_ascii_lowercase(), block_id);

        if is_item_only {
            if !reg.cached_creative_items.contains(&block_id) {
                reg.cached_creative_items.push(block_id);
            }
        } else if !reg.cached_creative_blocks.contains(&block_id) {
            reg.cached_creative_blocks.push(block_id);
        }

        Ok(())
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
                    pattern.push(Some(Self::resolve_identifier(s, reg)?));
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
        parse_vanilla_block_id(&clean)
    }
}
