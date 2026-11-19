//! Pokojowy przekaźnik TCP (relay) dla hostów za NAT z szyfrowaniem end-to-end ChaCha20-Poly1305.

mod crypto;
mod helpers;
mod server;
#[cfg(test)]
mod tests;

pub use helpers::{generate_room_code, invite_address, redact_invite_address};
pub use server::{connect_guest, run_relay_server, RelayServer, ReverseTunnelHost};
