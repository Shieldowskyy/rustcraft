//! Dedykowany oraz wbudowany serwer multiplayer (`run_dedicated_server`, `EmbeddedServer`, `ServerConfig`).
//! Całkowicie wolny od atrybutów `#[cfg]`.

mod client_io;
mod commands_chat;
mod commands_console;
mod config;
mod runner;
#[cfg(test)]
mod tests_a;
#[cfg(test)]
mod tests_b;
#[cfg(test)]
mod tests_c;

pub use config::{bind_server_listener, ServerConfig};
pub use runner::{run_dedicated_server, EmbeddedServer, EmbeddedServerState};
