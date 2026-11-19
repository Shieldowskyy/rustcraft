//! Silnik modów oparty na danych (`features::modding`): rejestr bloków/przedmiotów, parser JSON, synteza tekstur i komendy czatu.
//! Całkowicie wolny od atrybutów `#[cfg]` — ścieżki katalogu `mods/` delegowane do `crate::platform::storage::mods_dir()`.

mod commands;
mod json;
mod loader;
mod registry;
#[cfg(test)]
mod tests;
mod textures;
mod types;

pub use commands::execute_mod_command;
pub(crate) use json::parse_json_value_root;
pub use loader::ModManager;
pub use registry::{
    find_custom_block_by_name, find_mod_command, generate_mod_textures, get_all_mod_commands,
    get_custom_block_drop, get_custom_creative_blocks, get_custom_creative_items,
    get_custom_face_color, get_custom_mob_drops, get_custom_name, get_custom_nutrition,
    get_custom_smelting, get_custom_texture_id, is_custom_food, is_custom_item_only,
    is_custom_opaque, is_custom_solid, is_custom_transparent,
};
pub use types::{
    CustomBlockDef, ModAction, ModCommandDef, ModManifest, ModMobDrop, TexturePattern,
};
