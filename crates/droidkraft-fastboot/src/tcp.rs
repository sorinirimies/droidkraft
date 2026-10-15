//! Fastboot-over-TCP transport (Cuttlefish, emulators, `fastboot -s tcp:HOST`).
//!
//! Handshake: both sides send `FB01`. Every message is prefixed with an
//! 8-byte big-endian length.

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::transport::Transport;

pub struct Tcp(TcpStream);

impl Tcp {
    pub fn connect(addr: &str) -> Result<Self, String> {
        let mut s = TcpStream::connect(addr).map_err(|e| format!("connect {addr}: {e}"))?;
        s.write_all(b"FB01").map_err(|e| e.to_string())?;
        let mut hello = [0u8; 4];
        s.read_exact(&mut hello).map_err(|e| e.to_string())?;
        if &hello != b"FB01" {
            return Err(format!("bad handshake {hello:?}"));
        }
        Ok(Self(s))
    }
}

impl Transport for Tcp {
    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        let mut m = (data.len() as u64).to_be_bytes().to_vec();
        m.extend_from_slice(data);
        self.0.write_all(&m).map_err(|e| format!("tcp write: {e}"))
    }
    fn recv(&mut self) -> Result<Vec<u8>, String> {
        let mut len = [0u8; 8];
        self.0
            .read_exact(&mut len)
            .map_err(|e| format!("tcp read: {e}"))?;
        let mut buf = vec![0u8; u64::from_be_bytes(len) as usize];
        self.0
            .read_exact(&mut buf)
            .map_err(|e| format!("tcp read: {e}"))?;
        Ok(buf)
    }
    fn chunk_size(&self) -> usize {
        1024 * 1024
    }
}
