pub mod cli;
pub mod core;
pub mod gameplay;
pub mod graphics;
pub mod platform;
pub mod ui;
pub mod world;
pub mod app;

#[cfg(target_arch = "wasm32")]
pub mod web;
