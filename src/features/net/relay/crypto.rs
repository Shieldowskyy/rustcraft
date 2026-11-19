//! Szyfrowany strumień przekaźnika (`EncryptedStream`, ChaCha20-Poly1305 + HKDF-SHA256) oraz dwukierunkowe kopiowanie strumieni.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::thread;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;

const MAX_ENCRYPTED_RECORD_SIZE: usize = 64 * 1024;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;

pub(super) trait CloneableStream: Read + Write + Send + Sized + 'static {
    fn try_clone_stream(&self) -> io::Result<Self>;
    fn shutdown_write(&self) -> io::Result<()>;
}

pub(super) struct EncryptedStream {
    stream: TcpStream,
    transmit_cipher: ChaCha20Poly1305,
    receive_cipher: ChaCha20Poly1305,
    send_sequence: u64,
    receive_sequence: u64,
    decrypted: Vec<u8>,
    decrypted_offset: usize,
}

impl EncryptedStream {
    pub(super) fn new(stream: TcpStream, secret: [u8; 32], session_id: u64, is_host: bool) -> Self {
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

pub(super) fn copy_bidirectional<A: CloneableStream, B: CloneableStream>(
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
