//! Backwards-compatible facade re-exporting `crate::features::modding`.

pub(crate) use crate::features::modding::parse_json_value_root;
pub use crate::features::modding::{
    execute_mod_command, find_custom_block_by_name, find_mod_command, generate_mod_textures,
    get_all_mod_commands, get_custom_block_drop, get_custom_creative_blocks,
    get_custom_creative_items, get_custom_face_color, get_custom_mob_drops, get_custom_name,
    get_custom_nutrition, get_custom_smelting, get_custom_texture_id, is_custom_food,
    is_custom_item_only, is_custom_opaque, is_custom_solid, is_custom_transparent, CustomBlockDef,
    ModAction, ModCommandDef, ModManager, ModManifest, ModMobDrop, TexturePattern,
};
