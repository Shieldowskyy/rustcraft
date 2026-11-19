//! Fasada czatu w grze oraz konsoli komend (`/gamemode`, `/time`, `/give`, `/tp`, `/spawn`, `/weather`).
//! Całkowicie wolna od `#[cfg]`.

mod commands;
mod commands_world;
mod render;
mod selection;
mod state;
#[cfg(test)]
mod tests;

pub use commands::{parse_block_id, parse_vanilla_block_id};
pub use state::{ChatAction, ChatMessage, ChatState, CommandContext};
