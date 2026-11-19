//! Rendering domain facade (procedural `atlas`, `clouds`, `font`, `lighting`, `player_parts`, `software`, `traits`).
//! Strictly isolated from platform-specific `#[cfg]` code.

pub mod atlas;
pub mod clouds;
pub mod font;
pub mod lighting;
pub mod player_parts;
pub mod software;
pub mod traits;

pub use atlas::{Texture, TextureAtlas, TextureId, TextureRenderPass, TEX_SIZE};
pub use font::{draw_text, measure_text, GLYPH_H, GLYPH_W};
pub use lighting::{chunk_spawn_y_offset, compute_entity_light_rgb};
pub use player_parts::{emit_player_model_parts, PlayerBoxPart, PlayerHeldItemPose};
pub use software::{Particle, Renderer};
pub use traits::RenderBackend;
