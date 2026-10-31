//! Room-based TCP relay for hosts behind NAT.
//!
//! The relay only pairs an invited guest connection with an outbound connection
//! opened by the host. Game protocol bytes are forwarded without interpretation.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const ROOM_ID_LEN: usize = 32;
const ROOM_SECRET_LEN: usize = 64;
const MAX_INVITE_CODE_LEN: usize = ROOM_ID_LEN + 1 + ROOM_SECRET_LEN;
const MAX_CONTROL_LINE: usize = 128;
const MAX_GUESTS_PER_ROOM: usize = 20;
const MAX_RELAY_CONNECTIONS: usize = 256;
const PAIR_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ENCRYPTED_RECORD_SIZE: usize = 64 * 1024;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;

type RelayState = Arc<Mutex<HashMap<String, Room>>>;

struct Room {
    control: mpsc::Sender<u64>,
    pending: HashMap<u64, mpsc::SyncSender<TcpStream>>,
    guest_count: usize,
    next_session: u64,
}

/// A running public relay listener. Dropping it stops its accept loop.
pub struct RelayServer {
    local_addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

/// Starts a relay process that stays alive until the process is interrupted.
pub fn run_relay_server(bind_addr: &str) -> io::Result<()> {
    let server = RelayServer::bind(bind_addr)?;
    println!("[RELAY] Listening on {}", server.local_addr());
    loop {
        thread::park_timeout(Duration::from_secs(60));
    }
}

impl RelayServer {
    /// Starts a relay service on the given address (use `0.0.0.0:26657` for a VPS).
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

/// Keeps an outbound control connection to the relay and forwards guest sessions
/// to the local game server. The invitation secret stays between the host and guest.
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
        let room_for_thread = room_id;
        let secret_for_thread = secret;
        let thread = thread::Builder::new()
            .name("rustcraft-relay-host".to_string())
            .spawn(move || {
                host_control_loop(
                    stream,
                    addr,
                    room_for_thread,
                    secret_for_thread,
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

/// Opens a guest connection and returns the paired game-protocol stream.
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

/// Formats the address accepted by the multiplayer join field.
pub fn invite_address(relay_addr: &str, room_code: &str) -> Result<String, String> {
    parse_invite_code(room_code)?;
    if relay_addr.trim().is_empty()
        || relay_addr.contains('/')
        || relay_addr.contains(char::is_whitespace)
    {
        return Err("Invalid relay address".to_string());
    }
    Ok(format!("relay://{}/{room_code}", relay_addr.trim()))
}

pub fn redact_invite_address(address: &str) -> String {
    let Some(invite) = address.strip_prefix("relay://") else {
        return address.to_string();
    };
    match invite.split_once('/') {
        Some((relay_addr, _)) => format!("relay://{relay_addr}/<invite>"),
        None => "relay://<invalid invite>".to_string(),
    }
}

/// Creates an unpredictable invitation room code.
pub fn generate_room_code() -> Result<String, String> {
    let mut bytes = [0u8; 48];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| format!("Could not generate invitation code: {e}"))?;
    Ok(format!(
        "{}.{}",
        encode_hex(&bytes[..16]),
        encode_hex(&bytes[16..])
    ))
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
    let command = match read_control_line(&stream) {
        Ok(command) => command,
        Err(_) => return,
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
                    let secret = secret;
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

trait CloneableStream: Read + Write + Send + Sized + 'static {
    fn try_clone_stream(&self) -> io::Result<Self>;
    fn shutdown_write(&self) -> io::Result<()>;
}

struct EncryptedStream {
    stream: TcpStream,
    transmit_cipher: ChaCha20Poly1305,
    receive_cipher: ChaCha20Poly1305,
    send_sequence: u64,
    receive_sequence: u64,
    decrypted: Vec<u8>,
    decrypted_offset: usize,
}

impl EncryptedStream {
    fn new(stream: TcpStream, secret: [u8; 32], session_id: u64, is_host: bool) -> Self {
        let (transmit_label, receive_label) = if is_host {
            (b"host-to-guest".as_slice(), b"guest-to-host".as_slice())
        } else {
            (b"guest-to-host".as_slice(), b"host-to-guest".as_slice())
        };
        Self {
            stream,
            transmit_cipher: derive_cipher(&secret, session_id, transmit_label),
            receive_cipher: derive_cipher(&secret, session_id, receive_label),
            send_sequence: 0,
            receive_sequence: 0,
            decrypted: Vec::new(),
            decrypted_offset: 0,
        }
    }

    fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            stream: self.stream.try_clone()?,
            transmit_cipher: self.transmit_cipher.clone(),
            receive_cipher: self.receive_cipher.clone(),
            send_sequence: self.send_sequence,
            receive_sequence: self.receive_sequence,
            decrypted: Vec::new(),
            decrypted_offset: 0,
        })
    }

    fn read_record(&mut self) -> io::Result<()> {
        let mut length_bytes = [0u8; 4];
        self.stream.read_exact(&mut length_bytes)?;
        let record_len = u32::from_be_bytes(length_bytes) as usize;
        if !(NONCE_SIZE + TAG_SIZE..=MAX_ENCRYPTED_RECORD_SIZE + NONCE_SIZE + TAG_SIZE)
            .contains(&record_len)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid encrypted relay record length",
            ));
        }
        let mut record = vec![0u8; record_len];
        self.stream.read_exact(&mut record)?;
        let record_sequence = u64::from_be_bytes(record[4..12].try_into().unwrap());
        if record[..4] != [0; 4] || record_sequence != self.receive_sequence {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "encrypted relay record is out of sequence",
            ));
        }
        let plaintext = self
            .receive_cipher
            .decrypt(
                Nonce::from_slice(&record[..NONCE_SIZE]),
                &record[NONCE_SIZE..],
            )
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "relay record authentication failed",
                )
            })?;
        self.receive_sequence = self.receive_sequence.checked_add(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "relay sequence exhausted")
        })?;
        self.decrypted = plaintext;
        self.decrypted_offset = 0;
        Ok(())
    }
}

impl Read for EncryptedStream {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.decrypted_offset >= self.decrypted.len() {
            self.read_record()?;
        }
        let available = &self.decrypted[self.decrypted_offset..];
        let count = output.len().min(available.len());
        output[..count].copy_from_slice(&available[..count]);
        self.decrypted_offset += count;
        Ok(count)
    }
}

impl Write for EncryptedStream {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if input.is_empty() {
            return Ok(0);
        }
        let count = input.len().min(MAX_ENCRYPTED_RECORD_SIZE);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..].copy_from_slice(&self.send_sequence.to_be_bytes());
        let ciphertext = self
            .transmit_cipher
            .encrypt(Nonce::from_slice(&nonce), &input[..count])
            .map_err(|_| io::Error::other("could not encrypt relay record"))?;
        let record_len = NONCE_SIZE + ciphertext.len();
        let mut record = Vec::with_capacity(4 + record_len);
        record.extend_from_slice(&(record_len as u32).to_be_bytes());
        record.extend_from_slice(&nonce);
        record.extend_from_slice(&ciphertext);
        self.stream.write_all(&record)?;
        self.send_sequence = self
            .send_sequence
            .checked_add(1)
            .ok_or_else(|| io::Error::other("relay record sequence exhausted"))?;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

fn derive_cipher(secret: &[u8; 32], session_id: u64, direction: &[u8]) -> ChaCha20Poly1305 {
    let hkdf = Hkdf::<Sha256>::new(Some(b"rustcraft-relay-v1"), secret);
    let mut info = Vec::with_capacity(direction.len() + 8);
    info.extend_from_slice(direction);
    info.extend_from_slice(&session_id.to_be_bytes());
    let mut key = [0u8; 32];
    hkdf.expand(&info, &mut key)
        .expect("32-byte ChaCha20-Poly1305 key is a valid HKDF output length");
    ChaCha20Poly1305::new((&key).into())
}

impl CloneableStream for TcpStream {
    fn try_clone_stream(&self) -> io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> io::Result<()> {
        self.shutdown(Shutdown::Write)
    }
}

impl CloneableStream for EncryptedStream {
    fn try_clone_stream(&self) -> io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Write)
    }
}

fn copy_bidirectional<A: CloneableStream, B: CloneableStream>(
    left: &mut A,
    right: &mut B,
) -> io::Result<()> {
    let mut left_reader = left.try_clone_stream()?;
    let mut right_writer = right.try_clone_stream()?;
    let forward = thread::spawn(move || {
        let result = io::copy(&mut left_reader, &mut right_writer);
        let _ = right_writer.shutdown_write();
        result
    });

    let reverse_result = io::copy(right, left);
    let _ = left.shutdown_write();
    let forward_result = forward
        .join()
        .map_err(|_| io::Error::other("relay forwarding thread panicked"))?;
    reverse_result?;
    forward_result?;
    Ok(())
}

fn validate_room_id(room_id: &str) -> Result<(), String> {
    if room_id.len() != ROOM_ID_LEN || !room_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Invalid invitation room ID".to_string());
    }
    Ok(())
}

fn parse_invite_code(invite_code: &str) -> Result<(String, [u8; 32]), String> {
    if invite_code.len() != MAX_INVITE_CODE_LEN {
        return Err("Invitation code has an invalid length".to_string());
    }
    let (room_id, encoded_secret) = invite_code
        .split_once('.')
        .ok_or_else(|| "Invitation code must contain a '.' separator".to_string())?;
    validate_room_id(room_id)?;
    if encoded_secret.len() != ROOM_SECRET_LEN
        || !encoded_secret.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("Invitation secret is invalid".to_string());
    }
    let decoded = decode_hex(encoded_secret)?;
    let secret: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "Invitation secret is invalid".to_string())?;
    Ok((room_id.to_string(), secret))
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").expect("writing to a String cannot fail");
    }
    result
}

fn decode_hex(encoded: &str) -> Result<Vec<u8>, String> {
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                b'A'..=b'F' => Ok(byte - b'A' + 10),
                _ => Err("Invitation secret is invalid".to_string()),
            };
            Ok((digit(pair[0])? << 4) | digit(pair[1])?)
        })
        .collect()
}

fn resolve_addr(addr: &str) -> Result<SocketAddr, String> {
    addr.to_socket_addrs()
        .map_err(|e| format!("Invalid relay address '{addr}': {e}"))?
        .next()
        .ok_or_else(|| format!("Could not resolve relay address '{addr}'"))
}

fn write_control(stream: &TcpStream, message: &str) -> Result<(), String> {
    let mut writer = stream
        .try_clone()
        .map_err(|e| format!("Could not access relay connection: {e}"))?;
    writer
        .write_all(message.as_bytes())
        .map_err(|e| format!("Relay communication failed: {e}"))
}

fn read_control_line(stream: &TcpStream) -> Result<String, String> {
    let mut reader = stream
        .try_clone()
        .map_err(|e| format!("Could not access relay connection: {e}"))?;
    let mut line = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        reader
            .read_exact(&mut byte)
            .map_err(|e| format!("Relay communication failed: {e}"))?;
        if byte[0] == b'\n' {
            break;
        }
        if line.len() >= MAX_CONTROL_LINE {
            return Err("Relay response is too long".to_string());
        }
        line.push(byte[0]);
    }
    String::from_utf8(line).map_err(|_| "Relay response was not valid UTF-8".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_codes_are_unique_and_valid() {
        let first = generate_room_code().unwrap();
        let second = generate_room_code().unwrap();
        assert_eq!(first.len(), MAX_INVITE_CODE_LEN);
        assert_ne!(first, second);
        assert!(parse_invite_code(&first).is_ok());
        assert!(parse_invite_code("../bad").is_err());
        assert!(parse_invite_code("").is_err());
    }

    #[test]
    fn encrypted_stream_rejects_an_incorrect_invite_secret() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).unwrap();
        let (server, _) = listener.accept().unwrap();
        let mut sender = EncryptedStream::new(client, [1; 32], 7, false);
        let mut receiver = EncryptedStream::new(server, [2; 32], 7, true);

        sender.write_all(b"secret game packet").unwrap();
        let mut plaintext = [0u8; 19];
        let error = receiver.read_exact(&mut plaintext).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn relay_pairs_guest_stream_with_host_tunnel() {
        let relay = RelayServer::bind("127.0.0.1:0").unwrap();
        let relay_addr = relay.local_addr();
        let room_code = generate_room_code().unwrap();
        let host_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let host_addr = host_listener.local_addr().unwrap();
        let host_worker = thread::spawn(move || {
            let (mut local, _) = host_listener.accept().unwrap();
            let mut request = [0u8; 4];
            local.read_exact(&mut request).unwrap();
            assert_eq!(&request, b"ping");
            local.write_all(b"pong").unwrap();
        });

        let tunnel =
            ReverseTunnelHost::connect(&relay_addr.to_string(), room_code.clone(), host_addr)
                .unwrap();
        let mut guest = connect_guest(&relay_addr.to_string(), &room_code).unwrap();
        guest.write_all(b"ping").unwrap();
        let mut response = [0u8; 4];
        guest.read_exact(&mut response).unwrap();
        assert_eq!(&response, b"pong");
        host_worker.join().unwrap();
        drop(guest);
        drop(tunnel);
        drop(relay);
    }
}
