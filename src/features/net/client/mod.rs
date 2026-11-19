//! Nieblokujący klient sieciowy TCP do gry wieloosobowej (`NetworkClient`, `RemotePlayer`, `NetworkEvent`).

mod recv;
mod state;

pub use state::{NetworkClient, NetworkEvent, RemotePlayer};
