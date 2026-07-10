//! Background logcat streaming engine.
//!
//! [`LogcatStream`] spawns a background thread that connects to the local ADB
//! server, streams `logcat` from a device, and forwards complete lines over a
//! bounded channel.  Frontends drain the channel on their own tick loop.

use adb_client::ADBServerDevice;
use std::fmt;
use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::mpsc;

use crate::client::ADB_PORT;

/// Capacity of the bounded channel between the streaming thread and the UI.
/// When full, the background thread blocks, applying natural backpressure so
/// memory usage stays bounded during logcat bursts.
pub const CHANNEL_CAPACITY: usize = 10_000;

/// A [`Write`] that buffers bytes, splits on newlines, and sends each complete
/// line through an `mpsc::SyncSender<String>`.
pub struct ChannelWriter {
    sender: mpsc::SyncSender<String>,
    buffer: Vec<u8>,
}

impl ChannelWriter {
    pub fn new(sender: mpsc::SyncSender<String>) -> Self {
        Self {
            sender,
            buffer: Vec::with_capacity(4096),
        }
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.buffer.drain(..=newline_pos).collect();
            let line = String::from_utf8_lossy(&line_bytes)
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();
            if self.sender.send(line).is_err() {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "logcat receiver dropped",
                ));
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            let remaining = String::from_utf8_lossy(&self.buffer).to_string();
            self.buffer.clear();
            if !remaining.is_empty() {
                let _ = self.sender.send(remaining);
            }
        }
        Ok(())
    }
}

impl fmt::Debug for ChannelWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelWriter")
            .field("buffer_len", &self.buffer.len())
            .finish()
    }
}

/// Whether draining produced lines or the stream ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainStatus {
    /// The stream is still active.
    Active,
    /// The background thread disconnected; streaming has ended.
    Disconnected,
}

/// A background logcat stream from a single device.
#[derive(Debug, Default)]
pub struct LogcatStream {
    receiver: Option<mpsc::Receiver<String>>,
    running: bool,
}

impl LogcatStream {
    pub fn new() -> Self {
        Self {
            receiver: None,
            running: false,
        }
    }

    /// Whether the stream is (believed to be) running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Start streaming logcat from the device identified by `serial`.
    ///
    /// Any existing stream is stopped first.
    pub fn start(&mut self, serial: String) {
        self.stop();
        let (tx, rx) = mpsc::sync_channel::<String>(CHANNEL_CAPACITY);
        self.receiver = Some(rx);
        self.running = true;

        std::thread::spawn(move || {
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), ADB_PORT);
            let mut device = ADBServerDevice::new(serial, Some(addr));
            let writer = ChannelWriter::new(tx.clone());
            if let Err(e) = device.get_logs(writer) {
                let _ = tx.send(format!("--- LOGCAT ERROR: {} ---", e));
            }
        });
    }

    /// Stop the current stream by dropping the receiver.  The background
    /// thread's next send then fails with `BrokenPipe` and it exits.
    pub fn stop(&mut self) {
        self.receiver = None;
        self.running = false;
    }

    /// Drain up to `max` lines from the channel into `out`.
    ///
    /// Returns the number of lines drained and whether the stream is still
    /// active.  Never blocks.
    pub fn drain_into(&mut self, out: &mut Vec<String>, max: usize) -> (usize, DrainStatus) {
        let receiver = match &self.receiver {
            Some(rx) => rx,
            None => return (0, DrainStatus::Disconnected),
        };
        let mut count = 0;
        for _ in 0..max {
            match receiver.try_recv() {
                Ok(line) => {
                    out.push(line);
                    count += 1;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.running = false;
                    return (count, DrainStatus::Disconnected);
                }
            }
        }
        (count, DrainStatus::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_writer_splits_lines() {
        let (tx, rx) = mpsc::sync_channel(16);
        let mut w = ChannelWriter::new(tx);
        w.write_all(b"line one\nline two\npartial").unwrap();
        assert_eq!(rx.recv().unwrap(), "line one");
        assert_eq!(rx.recv().unwrap(), "line two");
        // "partial" has no newline yet — nothing more buffered as a line.
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn channel_writer_flush_sends_remainder() {
        let (tx, rx) = mpsc::sync_channel(16);
        let mut w = ChannelWriter::new(tx);
        w.write_all(b"tail").unwrap();
        w.flush().unwrap();
        assert_eq!(rx.recv().unwrap(), "tail");
    }

    #[test]
    fn stream_drain_empty_when_not_started() {
        let mut s = LogcatStream::new();
        let mut out = Vec::new();
        let (n, status) = s.drain_into(&mut out, 10);
        assert_eq!(n, 0);
        assert_eq!(status, DrainStatus::Disconnected);
        assert!(!s.is_running());
    }
}
