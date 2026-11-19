//! Rendering domain facade (procedural `atlas`, `clouds`, `font`, `software`, `traits`).
//! Strictly isolated from platform-specific `#[cfg]` code.

pub mod atlas;
pub mod clouds;
pub mod font;
pub mod software;
pub mod traits;

pub use atlas::{Texture, TextureAtlas, TextureId, TEX_SIZE};
pub use font::{draw_text, measure_text, GLYPH_H, GLYPH_W};
pub use software::{Particle, Renderer};
pub use traits::RenderBackend;
