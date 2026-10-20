//! Multiplayer client-server networking modules for RustCraft.

pub mod client;
pub mod discovery;
pub mod protocol;
pub mod server;
pub mod server_log;

pub use client::{NetworkClient, NetworkEvent, RemotePlayer};
pub use discovery::{DiscoveredServer, LanDiscoveryClient, LanDiscoveryHost, LAN_DISCOVERY_PORT};
pub use protocol::{ClientPacket, PlayerInfo, ServerPacket, DEFAULT_SERVER_PORT, PROTOCOL_VERSION};
pub use server::{bind_server_listener, run_dedicated_server};
pub use server_log::{init_server_logger, shutdown_server_logger, server_log};
