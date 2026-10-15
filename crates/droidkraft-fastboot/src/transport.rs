//! Byte-level fastboot transport (USB, TCP, or a test double).

pub trait Transport {
    /// Send raw bytes (a command or a download chunk).
    fn send(&mut self, data: &[u8]) -> Result<(), String>;
    /// Receive one response packet.
    fn recv(&mut self) -> Result<Vec<u8>, String>;
    /// Preferred download chunk size.
    fn chunk_size(&self) -> usize;
}
