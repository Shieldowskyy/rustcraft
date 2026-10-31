//! Multiplayer client-server networking modules for RustCraft.

pub mod client;
pub mod discovery;
pub mod protocol;
pub mod relay;
pub mod server;
pub mod server_log;

pub use client::{NetworkClient, NetworkEvent, RemotePlayer};
pub use discovery::{DiscoveredServer, LanDiscoveryClient, LanDiscoveryHost, LAN_DISCOVERY_PORT};
pub use protocol::{ClientPacket, PlayerInfo, ServerPacket, DEFAULT_SERVER_PORT, PROTOCOL_VERSION};
pub use relay::{
    connect_guest as connect_relay_guest, generate_room_code, invite_address,
    redact_invite_address, run_relay_server, RelayServer, ReverseTunnelHost,
};
pub use server::{
    bind_server_listener, run_dedicated_server, EmbeddedServer, EmbeddedServerState,
};
pub use server_log::{init_server_logger, shutdown_server_logger, server_log};
