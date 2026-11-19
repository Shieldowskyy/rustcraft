//! Sony PS Vita hardware `libgxm` (PowerVR SGX543MP4+) 3D renderer facade.
//! Isolates GXM FFI, GPU memory, Cg/GXP shaders, texture atlas, terrain/entity passes, and frustum culling.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

mod entities;
mod ffi;
mod frustum;
mod init;
mod mem;
mod models;
mod programs;
mod render;
mod render_dynamic;
mod render_world;
mod renderer;
mod textures;
mod types;

pub use renderer::GxmRenderer;
pub use types::{GxmPackedVertex, GxmVertex};
