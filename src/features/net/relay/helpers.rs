//! Generowanie i walidacja kodów zaproszeń przekaźnika (`room_id.secret`), formatowanie adresów `relay://` i pomocnicze funkcje kontrolne TCP.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub(super) const ROOM_ID_LEN: usize = 32;
pub(super) const ROOM_SECRET_LEN: usize = 64;
pub(super) const MAX_INVITE_CODE_LEN: usize = ROOM_ID_LEN + 1 + ROOM_SECRET_LEN;
pub(super) const MAX_CONTROL_LINE: usize = 128;
pub(super) const MAX_GUESTS_PER_ROOM: usize = 20;
pub(super) const MAX_RELAY_CONNECTIONS: usize = 256;
pub(super) const PAIR_TIMEOUT: Duration = Duration::from_secs(10);

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

pub(super) fn validate_room_id(room_id: &str) -> Result<(), String> {
    if room_id.len() != ROOM_ID_LEN || !room_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Invalid invitation room ID".to_string());
    }
    Ok(())
}

pub(super) fn parse_invite_code(invite_code: &str) -> Result<(String, [u8; 32]), String> {
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

pub(super) fn resolve_addr(addr: &str) -> Result<SocketAddr, String> {
    addr.to_socket_addrs()
        .map_err(|e| format!("Invalid relay address '{addr}': {e}"))?
        .next()
        .ok_or_else(|| format!("Could not resolve relay address '{addr}'"))
}

pub(super) fn write_control(stream: &TcpStream, message: &str) -> Result<(), String> {
    let mut writer = stream
        .try_clone()
        .map_err(|e| format!("Could not access relay connection: {e}"))?;
    writer
        .write_all(message.as_bytes())
        .map_err(|e| format!("Relay communication failed: {e}"))
}

pub(super) fn read_control_line(stream: &TcpStream) -> Result<String, String> {
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
