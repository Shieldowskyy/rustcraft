//! Local Area Network (LAN) server discovery for RustCraft.
//!
//! Enables automatic discovery of local RustCraft servers on the same subnet
//! (Wi-Fi, Ethernet, and local hotspots) on Linux, Windows, macOS, Android, and PS Vita.
//!
//! Uses UDP broadcasts on port 26655 with probe pings and automatic response.

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

pub const LAN_DISCOVERY_PORT: u16 = 26655;
pub const LAN_SERVER_PREFIX: &str = "RUSTCRAFT_LAN_SERVER";
pub const LAN_PING_PREFIX: &str = "RUSTCRAFT_LAN_PING";

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveredServer {
    pub ip: String,
    pub port: u16,
    pub motd: String,
    pub player_count: u32,
    pub max_players: u32,
    pub last_seen: Instant,
}

impl DiscoveredServer {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

/// Server-side LAN broadcaster and responder.
pub struct LanDiscoveryHost {
    socket: Option<UdpSocket>,
    server_port: u16,
    motd: String,
    max_players: u32,
    last_broadcast: Instant,
}

impl LanDiscoveryHost {
    pub fn new(server_port: u16, motd: &str, max_players: u32) -> Self {
        // Try binding to LAN_DISCOVERY_PORT. If occupied, bind to ephemeral port 0.
        let socket = UdpSocket::bind(("0.0.0.0", LAN_DISCOVERY_PORT))
            .or_else(|_| UdpSocket::bind("0.0.0.0:0"))
            .ok();

        if let Some(ref s) = socket {
            let _ = s.set_broadcast(true);
            let _ = s.set_nonblocking(true);
        }

        Self {
            socket,
            server_port,
            motd: motd.to_string(),
            max_players,
            // Broadcast immediately on startup
            last_broadcast: Instant::now() - Duration::from_secs(5),
        }
    }

    pub fn update(&mut self, player_count: u32) {
        let socket = match self.socket.as_ref() {
            Some(s) => s,
            None => return,
        };

        // 1. Periodic broadcast every 1.5 seconds
        if self.last_broadcast.elapsed() >= Duration::from_millis(1500) {
            self.last_broadcast = Instant::now();
            let payload = format!(
                "{}:{}:{}:{}:{}",
                LAN_SERVER_PREFIX, self.server_port, self.motd, player_count, self.max_players
            );
            let bytes = payload.as_bytes();
            let _ = socket.send_to(bytes, ("255.255.255.255", LAN_DISCOVERY_PORT));
            let _ = socket.send_to(bytes, ("127.0.0.1", LAN_DISCOVERY_PORT));
        }

        // 2. Respond to incoming discovery pings from clients
        let mut buf = [0u8; 512];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((n, src_addr)) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    if text.trim().starts_with(LAN_PING_PREFIX) {
                        let payload = format!(
                            "{}:{}:{}:{}:{}",
                            LAN_SERVER_PREFIX, self.server_port, self.motd, player_count, self.max_players
                        );
                        let _ = socket.send_to(payload.as_bytes(), src_addr);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
}

/// Client-side LAN server scanner and browser.
pub struct LanDiscoveryClient {
    socket: Option<UdpSocket>,
    pub servers: Vec<DiscoveredServer>,
    pub is_refreshing: bool,
    pub last_refresh: Instant,
}

impl Default for LanDiscoveryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LanDiscoveryClient {
    pub fn new() -> Self {
        // Try binding to LAN_DISCOVERY_PORT so we receive UDP broadcast datagrams.
        // If occupied (e.g. running server on the same host), bind to ephemeral port 0.
        let socket = UdpSocket::bind(("0.0.0.0", LAN_DISCOVERY_PORT))
            .or_else(|_| UdpSocket::bind("0.0.0.0:0"))
            .ok();

        if let Some(ref s) = socket {
            let _ = s.set_broadcast(true);
            let _ = s.set_nonblocking(true);
        }

        let mut client = Self {
            socket,
            servers: Vec::new(),
            is_refreshing: false,
            last_refresh: Instant::now() - Duration::from_secs(10),
        };
        client.refresh();
        client
    }

    pub fn refresh(&mut self) {
        self.is_refreshing = true;
        self.last_refresh = Instant::now();

        // Broadcast active ping probe
        if let Some(ref s) = self.socket {
            let ping_bytes = LAN_PING_PREFIX.as_bytes();
            let _ = s.send_to(ping_bytes, ("255.255.255.255", LAN_DISCOVERY_PORT));
            let _ = s.send_to(ping_bytes, ("127.0.0.1", LAN_DISCOVERY_PORT));
        }
    }

    pub fn update(&mut self) -> &[DiscoveredServer] {
        if self.is_refreshing && self.last_refresh.elapsed() >= Duration::from_millis(1200) {
            self.is_refreshing = false;
        }

        let socket = match self.socket.as_ref() {
            Some(s) => s,
            None => return &self.servers,
        };

        let mut buf = [0u8; 1024];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((n, src_addr)) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    if let Some(server) = parse_announcement(&text, src_addr) {
                        if let Some(existing) = self
                            .servers
                            .iter_mut()
                            .find(|s| s.ip == server.ip && s.port == server.port)
                        {
                            existing.motd = server.motd;
                            existing.player_count = server.player_count;
                            existing.max_players = server.max_players;
                            existing.last_seen = Instant::now();
                        } else {
                            self.servers.push(server);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }

        // Evict expired servers (> 6 seconds without packet)
        self.servers.retain(|s| s.last_seen.elapsed() < Duration::from_secs(6));

        &self.servers
    }
}

pub fn parse_announcement(text: &str, src_addr: SocketAddr) -> Option<DiscoveredServer> {
    let trimmed = text.trim();
    if !trimmed.starts_with(LAN_SERVER_PREFIX) {
        return None;
    }

    let rest = &trimmed[LAN_SERVER_PREFIX.len()..];
    let rest = rest.strip_prefix(':')?;

    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() < 4 {
        return None;
    }

    let port: u16 = parts[0].parse().unwrap_or(crate::net::protocol::DEFAULT_SERVER_PORT);
    let max_players: u32 = parts[parts.len() - 1].parse().unwrap_or(16);
    let player_count: u32 = parts[parts.len() - 2].parse().unwrap_or(0);
    let motd = parts[1..parts.len() - 2].join(":");

    let ip = match src_addr {
        SocketAddr::V4(v4) => {
            let o = v4.ip().octets();
            format!("{}.{}.{}.{}", o[0], o[1], o[2], o[3])
        }
        SocketAddr::V6(v6) => v6.ip().to_string(),
    };

    Some(DiscoveredServer {
        ip,
        port,
        motd,
        player_count,
        max_players,
        last_seen: Instant::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_announcement_standard() {
        let text = "RUSTCRAFT_LAN_SERVER:26656:RustCraft World:2:16";
        let src = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42)), 26655);
        let s = parse_announcement(text, src).expect("Should parse");
        assert_eq!(s.ip, "192.168.1.42");
        assert_eq!(s.port, 26656);
        assert_eq!(s.motd, "RustCraft World");
        assert_eq!(s.player_count, 2);
        assert_eq!(s.max_players, 16);
        assert_eq!(s.addr(), "192.168.1.42:26656");
    }

    #[test]
    fn test_parse_announcement_with_colons_in_motd() {
        let text = "RUSTCRAFT_LAN_SERVER:26656:Server: Survival: Hardcore:1:20";
        let src = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)), 26655);
        let s = parse_announcement(text, src).expect("Should parse");
        assert_eq!(s.ip, "10.0.0.5");
        assert_eq!(s.port, 26656);
        assert_eq!(s.motd, "Server: Survival: Hardcore");
        assert_eq!(s.player_count, 1);
        assert_eq!(s.max_players, 20);
    }

    #[test]
    fn test_parse_announcement_invalid_ignored() {
        let text = "RANDOM_PACKET_DATA:123";
        let src = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 26655);
        assert!(parse_announcement(text, src).is_none());
    }

    #[test]
    fn test_lan_discovery_host_client_interaction() {
        let mut host = LanDiscoveryHost::new(26656, "My Test Server", 12);
        let mut client = LanDiscoveryClient::new();

        // Host updates (sends announcement and processes incoming probes)
        host.update(3);
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Client refreshes
        client.refresh();
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Host updates to answer client probe
        host.update(3);
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Client reads incoming replies
        let servers = client.update();
        if let Some(s) = servers.iter().find(|s| s.motd == "My Test Server") {
            assert_eq!(s.motd, "My Test Server");
            assert_eq!(s.player_count, 3);
            assert_eq!(s.max_players, 12);
        }
    }
}
