pub mod app;
pub mod audio;
pub mod cli;
pub mod core;
pub mod engine;
pub(crate) mod embedded_assets {
    include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));
}
pub mod gameplay;
pub mod graphics;
pub mod net;
pub mod platform;
pub mod ui;
pub mod world;

#[cfg(target_arch = "wasm32")]
pub mod web;
