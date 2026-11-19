//! Serwer przekaźnika TCP (`RelayServer`), tunel zwrotny hosta (`ReverseTunnelHost`) oraz mostek połączenia gościa (`connect_guest`).

use std::collections::HashMap;
use std::io::{self, Read};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::crypto::{copy_bidirectional, EncryptedStream};
use super::helpers::{
    parse_invite_code, read_control_line, resolve_addr, validate_room_id, write_control,
    MAX_CONTROL_LINE, MAX_GUESTS_PER_ROOM, MAX_RELAY_CONNECTIONS, PAIR_TIMEOUT,
};

type RelayState = Arc<Mutex<HashMap<String, Room>>>;

struct Room {
    control: mpsc::Sender<u64>,
    pending: HashMap<u64, mpsc::SyncSender<TcpStream>>,
    guest_count: usize,
    next_session: u64,
}

pub struct RelayServer {
    local_addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

pub fn run_relay_server(bind_addr: &str) -> io::Result<()> {
    let server = RelayServer::bind(bind_addr)?;
    println!("[RELAY] Listening on {}", server.local_addr());
    loop {
        thread::park_timeout(Duration::from_secs(60));
    }
}

impl RelayServer {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let local_addr = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let thread = thread::Builder::new()
            .name("rustcraft-relay".to_string())
            .spawn(move || serve(listener, stop_thread))?;

        Ok(Self {
            local_addr,
            stop,
            thread: Some(thread),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for RelayServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct ReverseTunnelHost {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ReverseTunnelHost {
    pub fn connect(
        relay_addr: &str,
        room_code: String,
        local_server_addr: SocketAddr,
    ) -> Result<Self, String> {
        let (room_id, secret) = parse_invite_code(&room_code)?;
        let addr = resolve_addr(relay_addr)?;
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
            .map_err(|e| format!("Could not connect to relay {addr}: {e}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("Could not configure relay connection: {e}"))?;
        write_control(&stream, &format!("HOST {room_id}\n"))?;
        let response = read_control_line(&stream)?;
        if response != "OK" {
            return Err(format!("Relay refused host registration: {response}"));
        }
        stream
            .set_read_timeout(None)
            .map_err(|e| format!("Could not configure relay connection: {e}"))?;
        stream
            .set_nonblocking(true)
            .map_err(|e| format!("Could not configure relay connection: {e}"))?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let thread = thread::Builder::new()
            .name("rustcraft-relay-host".to_string())
            .spawn(move || {
                host_control_loop(
                    stream,
                    addr,
                    room_id,
                    secret,
                    local_server_addr,
                    stop_thread,
                )
            })
            .map_err(|e| format!("Could not start relay host thread: {e}"))?;

        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for ReverseTunnelHost {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub fn connect_guest(relay_addr: &str, room_code: &str) -> Result<TcpStream, String> {
    let (room_id, secret) = parse_invite_code(room_code)?;
    let addr = resolve_addr(relay_addr)?;
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("Could not connect to relay {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(PAIR_TIMEOUT))
        .map_err(|e| format!("Could not configure relay connection: {e}"))?;
    write_control(&stream, &format!("JOIN {room_id}\n"))?;
    let response = read_control_line(&stream)?;
    let session_id = response
        .strip_prefix("READY ")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| format!("Relay could not connect you to the host: {response}"))?;
    if session_id == 0 {
        return Err(format!(
            "Relay could not connect you to the host: {response}"
        ));
    }
    stream
        .set_read_timeout(None)
        .map_err(|e| format!("Could not configure relay connection: {e}"))?;
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .map_err(|e| format!("Could not create local relay bridge: {e}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("Could not determine local relay bridge address: {e}"))?;
    let local_client = TcpStream::connect_timeout(&local_addr, Duration::from_secs(2))
        .map_err(|e| format!("Could not open local relay bridge: {e}"))?;
    let (local_bridge, _) = listener
        .accept()
        .map_err(|e| format!("Could not accept local relay bridge: {e}"))?;
    thread::Builder::new()
        .name("rustcraft-relay-guest".to_string())
        .spawn(move || {
            let mut encrypted = EncryptedStream::new(stream, secret, session_id, false);
            let mut local = local_bridge;
            let _ = copy_bidirectional(&mut encrypted, &mut local);
        })
        .map_err(|e| format!("Could not start relay guest thread: {e}"))?;
    Ok(local_client)
}

fn serve(listener: TcpListener, stop: Arc<AtomicBool>) {
    let rooms: RelayState = Arc::new(Mutex::new(HashMap::new()));
    let active_connections = Arc::new(AtomicUsize::new(0));
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                if active_connections.fetch_add(1, Ordering::AcqRel) >= MAX_RELAY_CONNECTIONS {
                    active_connections.fetch_sub(1, Ordering::AcqRel);
                    let _ = stream.shutdown(Shutdown::Both);
                    continue;
                }
                let rooms = Arc::clone(&rooms);
                let connection_count = Arc::clone(&active_connections);
                let result = thread::Builder::new()
                    .name("rustcraft-relay-connection".to_string())
                    .spawn(move || {
                        let _guard = RelayConnectionGuard(connection_count);
                        handle_connection(stream, rooms);
                    });
                if result.is_err() {
                    active_connections.fetch_sub(1, Ordering::AcqRel);
                }
            }
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => {
                eprintln!("[RELAY] Accept failed: {err}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

struct RelayConnectionGuard(Arc<AtomicUsize>);

impl Drop for RelayConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn handle_connection(stream: TcpStream, rooms: RelayState) {
    if stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .is_err()
    {
        return;
    }
    let Ok(command) = read_control_line(&stream) else {
        return;
    };
    let mut parts = command.split_whitespace();
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("HOST"), Some(room_id), None, None) => {
            if validate_room_id(room_id).is_err() {
                let _ = write_control(&stream, "ERR invalid room code\n");
                return;
            }
            handle_host(stream, rooms, room_id.to_string());
        }
        (Some("JOIN"), Some(room_id), None, None) => {
            if validate_room_id(room_id).is_err() {
                let _ = write_control(&stream, "ERR invalid room code\n");
                return;
            }
            handle_guest(stream, rooms, room_id.to_string());
        }
        (Some("DATA"), Some(room_id), Some(session), None) => {
            let Ok(session_id) = session.parse::<u64>() else {
                let _ = write_control(&stream, "ERR invalid session\n");
                return;
            };
            handle_data(stream, rooms, room_id.to_string(), session_id);
        }
        _ => {
            let _ = write_control(&stream, "ERR invalid request\n");
        }
    }
}

fn handle_host(mut stream: TcpStream, rooms: RelayState, room_code: String) {
    let (control_tx, control_rx) = mpsc::channel();
    {
        let Ok(mut rooms) = rooms.lock() else {
            let _ = write_control(&stream, "ERR relay unavailable\n");
            return;
        };
        if rooms.contains_key(&room_code) {
            let _ = write_control(&stream, "ERR room already hosted\n");
            return;
        }
        rooms.insert(
            room_code.clone(),
            Room {
                control: control_tx,
                pending: HashMap::new(),
                guest_count: 0,
                next_session: 1,
            },
        );
    }

    if write_control(&stream, "OK\n").is_err() {
        remove_room(&rooms, &room_code);
        return;
    }
    let _ = stream.set_nonblocking(true);
    let mut input = Vec::new();
    let mut buf = [0u8; 128];

    loop {
        loop {
            match control_rx.try_recv() {
                Ok(session_id) => {
                    if write_control(&stream, &format!("OPEN {session_id}\n")).is_err() {
                        remove_room(&rooms, &room_code);
                        return;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    remove_room(&rooms, &room_code);
                    return;
                }
            }
        }

        match stream.read(&mut buf) {
            Ok(0) => {
                remove_room(&rooms, &room_code);
                return;
            }
            Ok(n) => {
                input.extend_from_slice(&buf[..n]);
                if input.len() > MAX_CONTROL_LINE {
                    remove_room(&rooms, &room_code);
                    return;
                }
                while let Some(newline) = input.iter().position(|byte| *byte == b'\n') {
                    let line = String::from_utf8_lossy(&input[..newline]).into_owned();
                    input.drain(..=newline);
                    if line == "PING" && write_control(&stream, "PONG\n").is_err() {
                        remove_room(&rooms, &room_code);
                        return;
                    }
                }
            }
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {}
            Err(ref err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => {
                remove_room(&rooms, &room_code);
                return;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn handle_guest(mut guest: TcpStream, rooms: RelayState, room_code: String) {
    let (session_id, receiver) = {
        let Ok(mut rooms) = rooms.lock() else {
            let _ = write_control(&guest, "ERR relay unavailable\n");
            return;
        };
        let Some(room) = rooms.get_mut(&room_code) else {
            let _ = write_control(&guest, "ERR host is not online\n");
            return;
        };
        if room.guest_count >= MAX_GUESTS_PER_ROOM {
            let _ = write_control(&guest, "ERR room is full\n");
            return;
        }
        let session_id = room.next_session;
        room.next_session = room.next_session.wrapping_add(1).max(1);
        let (sender, receiver) = mpsc::sync_channel(1);
        room.pending.insert(session_id, sender);
        room.guest_count += 1;
        if room.control.send(session_id).is_err() {
            room.pending.remove(&session_id);
            room.guest_count -= 1;
            let _ = write_control(&guest, "ERR host disconnected\n");
            return;
        }
        (session_id, receiver)
    };

    let Ok(mut host) = receiver.recv_timeout(PAIR_TIMEOUT) else {
        remove_pending_guest(&rooms, &room_code, session_id);
        let _ = write_control(&guest, "ERR host did not accept the connection\n");
        return;
    };

    if write_control(&guest, &format!("READY {session_id}\n")).is_err() {
        remove_guest(&rooms, &room_code);
        return;
    }
    let _ = guest.set_nodelay(true);
    let _ = host.set_nodelay(true);
    let _ = copy_bidirectional(&mut guest, &mut host);
    remove_guest(&rooms, &room_code);
}

fn handle_data(stream: TcpStream, rooms: RelayState, room_code: String, session_id: u64) {
    let sender = {
        let Ok(mut rooms) = rooms.lock() else {
            let _ = write_control(&stream, "ERR relay unavailable\n");
            return;
        };
        let Some(room) = rooms.get_mut(&room_code) else {
            let _ = write_control(&stream, "ERR host is not online\n");
            return;
        };
        room.pending.remove(&session_id)
    };
    let Some(sender) = sender else {
        let _ = write_control(&stream, "ERR no pending guest\n");
        return;
    };
    let _ = stream.set_read_timeout(None);
    let _ = stream.set_write_timeout(None);
    let _ = sender.send(stream);
}

fn remove_pending_guest(rooms: &RelayState, room_code: &str, session_id: u64) {
    if let Ok(mut rooms) = rooms.lock() {
        if let Some(room) = rooms.get_mut(room_code) {
            if room.pending.remove(&session_id).is_some() {
                room.guest_count = room.guest_count.saturating_sub(1);
            }
        }
    }
}

fn remove_guest(rooms: &RelayState, room_code: &str) {
    if let Ok(mut rooms) = rooms.lock() {
        if let Some(room) = rooms.get_mut(room_code) {
            room.guest_count = room.guest_count.saturating_sub(1);
        }
    }
}

fn remove_room(rooms: &RelayState, room_code: &str) {
    if let Ok(mut rooms) = rooms.lock() {
        rooms.remove(room_code);
    }
}

fn host_control_loop(
    mut control: TcpStream,
    relay_addr: SocketAddr,
    room_id: String,
    secret: [u8; 32],
    local_server_addr: SocketAddr,
    stop: Arc<AtomicBool>,
) {
    let mut input = Vec::new();
    let mut buf = [0u8; 128];
    let mut last_keepalive = std::time::Instant::now();
    while !stop.load(Ordering::Acquire) {
        if last_keepalive.elapsed() >= Duration::from_secs(15) {
            if write_control(&control, "PING\n").is_err() {
                return;
            }
            last_keepalive = std::time::Instant::now();
        }
        match control.read(&mut buf) {
            Ok(0) => return,
            Ok(n) => {
                input.extend_from_slice(&buf[..n]);
                if input.len() > MAX_CONTROL_LINE {
                    return;
                }
                while let Some(newline) = input.iter().position(|byte| *byte == b'\n') {
                    let line = String::from_utf8_lossy(&input[..newline]).into_owned();
                    input.drain(..=newline);
                    let Some(session_id) = line
                        .strip_prefix("OPEN ")
                        .and_then(|value| value.parse::<u64>().ok())
                    else {
                        continue;
                    };
                    let room_id = room_id.clone();
                    thread::spawn(move || {
                        open_host_session(
                            relay_addr,
                            &room_id,
                            &secret,
                            session_id,
                            local_server_addr,
                        )
                    });
                }
            }
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {}
            Err(ref err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return,
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _ = control.shutdown(Shutdown::Both);
}

fn open_host_session(
    relay_addr: SocketAddr,
    room_id: &str,
    secret: &[u8; 32],
    session_id: u64,
    local_server_addr: SocketAddr,
) {
    let Ok(relay) = TcpStream::connect_timeout(&relay_addr, Duration::from_secs(5)) else {
        return;
    };
    if write_control(&relay, &format!("DATA {room_id} {session_id}\n")).is_err() {
        return;
    }
    let Ok(mut local) = TcpStream::connect_timeout(&local_server_addr, Duration::from_secs(5))
    else {
        let _ = relay.shutdown(Shutdown::Both);
        return;
    };
    let _ = relay.set_nodelay(true);
    let _ = local.set_nodelay(true);
    let mut encrypted = EncryptedStream::new(relay, *secret, session_id, true);
    let _ = copy_bidirectional(&mut encrypted, &mut local);
}
