use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use super::crypto::EncryptedStream;
use super::helpers::{generate_room_code, parse_invite_code, MAX_INVITE_CODE_LEN};
use super::server::{connect_guest, RelayServer, ReverseTunnelHost};

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
        ReverseTunnelHost::connect(&relay_addr.to_string(), room_code.clone(), host_addr).unwrap();
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
