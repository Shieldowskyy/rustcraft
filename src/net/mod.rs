//! Backwards-compatible facade re-exporting `crate::features::net`.

pub mod client;
pub mod discovery;
pub mod protocol;
pub mod relay;
pub mod server;
pub mod server_log;

pub use crate::features::net::{
    bind_server_listener, connect_relay_guest, generate_room_code, init_server_logger,
    invite_address, redact_invite_address, run_dedicated_server, run_relay_server,
    shutdown_server_logger, ClientPacket, DiscoveredServer, EmbeddedServer, EmbeddedServerState,
    LanDiscoveryClient, LanDiscoveryHost, NetworkClient, NetworkEvent, PlayerInfo, RelayServer,
    RemotePlayer, ReverseTunnelHost, ServerPacket, DEFAULT_SERVER_PORT, LAN_DISCOVERY_PORT,
    PROTOCOL_VERSION,
};
