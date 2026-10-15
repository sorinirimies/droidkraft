//! Native fastboot protocol over USB — no external binary.
//!
//! Protocol: the host sends an ASCII command; the device answers with
//! `INFO…`/`TEXT…` (progress), then `OKAY…`, `FAIL…`, or `DATA<hex len>`
//! (ready to receive a download).

use crate::transport::Transport;
use crate::types::{FastbootCommand, FastbootError, FastbootResult};
use crate::usb::Usb;

/// Why the native backend did not (fully) handle a command.
pub enum Outcome {
    /// Nothing was sent to the device; safe to retry with another backend.
    Fallback(FastbootError),
    /// The command ran (or was attempted); report this result as final.
    Done(FastbootResult<String>),
}

enum Reply {
    Okay(String),
    Fail(String),
    Data(usize),
}

fn read_reply(usb: &mut dyn Transport, info: &mut Vec<String>) -> Result<Reply, String> {
    loop {
        let pkt = usb.recv()?;
        if pkt.len() < 4 {
            return Err(format!("short response: {pkt:?}"));
        }
        let (tag, rest) = pkt.split_at(4);
        let text = String::from_utf8_lossy(rest).into_owned();
        match tag {
            b"INFO" | b"TEXT" => info.push(text),
            b"OKAY" => return Ok(Reply::Okay(text)),
            b"FAIL" => return Ok(Reply::Fail(text)),
            b"DATA" => {
                let n = usize::from_str_radix(text.trim(), 16)
                    .map_err(|_| format!("bad DATA length {text:?}"))?;
                return Ok(Reply::Data(n));
            }
            _ => {
                return Err(format!(
                    "unknown response {:?}",
                    String::from_utf8_lossy(&pkt)
                ))
            }
        }
    }
}

/// Send `cmd` and wait for OKAY; INFO lines are returned joined by newlines.
fn run(usb: &mut dyn Transport, cmd: &str) -> FastbootResult<String> {
    usb.send(cmd.as_bytes()).map_err(FastbootError::Usb)?;
    let mut info = Vec::new();
    match read_reply(usb, &mut info).map_err(FastbootError::Usb)? {
        Reply::Okay(msg) => {
            if !msg.is_empty() {
                info.push(msg);
            }
            Ok(info.join("\n"))
        }
        Reply::Fail(msg) => Err(FastbootError::CommandFailed(format!("FAILED ({msg})"))),
        Reply::Data(_) => Err(FastbootError::Usb("unexpected DATA response".into())),
    }
}

fn max_download(usb: &mut dyn Transport) -> Option<u64> {
    let v = run(usb, "getvar:max-download-size").ok()?;
    let v = v.trim().trim_start_matches("0x");
    u64::from_str_radix(v, 16).ok()
}

/// Download one buffer (`download:<len>` + data + OKAY).
fn download<R: std::io::Read + std::io::Seek>(
    usb: &mut dyn Transport,
    piece: &crate::sparse::Piece,
    src: &mut R,
) -> FastbootResult<()> {
    let len = piece.len();
    usb.send(format!("download:{len:08x}").as_bytes())
        .map_err(FastbootError::Usb)?;
    let mut info = Vec::new();
    match read_reply(usb, &mut info).map_err(FastbootError::Usb)? {
        Reply::Data(n) if n as u64 == len => {}
        Reply::Fail(m) => return Err(FastbootError::CommandFailed(format!("FAILED ({m})"))),
        _ => return Err(FastbootError::Usb("device refused download".into())),
    }
    let chunk = usb.chunk_size();
    crate::sparse::stream(piece, src, chunk, &mut |d| usb.send(d)).map_err(FastbootError::Usb)?;
    match read_reply(usb, &mut info).map_err(FastbootError::Usb)? {
        Reply::Okay(_) => Ok(()),
        Reply::Fail(m) => Err(FastbootError::CommandFailed(format!("FAILED ({m})"))),
        Reply::Data(_) => Err(FastbootError::Usb("unexpected DATA".into())),
    }
}

/// Flash `path` to `partition`; images over `max-download-size` are split into
/// sparse pieces, each downloaded and flashed in turn (like `fastboot flash`).
pub(crate) fn flash(
    usb: &mut dyn Transport,
    partition: &str,
    path: &str,
) -> FastbootResult<String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| FastbootError::CommandFailed(format!("read {path}: {e}")))?;
    let max = max_download(usb);
    let pieces = crate::sparse::plan(&mut file, max)
        .map_err(|e| FastbootError::CommandFailed(format!("sparse {path}: {e}")))?;
    let mut out = String::new();
    for piece in &pieces {
        download(usb, piece, &mut file)?;
        out = run(usb, &format!("flash:{partition}"))?;
    }
    Ok(out)
}

/// `fastboot -w`: erase userdata, then metadata.
///
/// Native version always uses `erase:` (the host tool prefers `format:` when
/// the device reports a known filesystem; `erase:` is accepted by all
/// bootloaders). A missing/unerasable `metadata` partition is ignored.
fn wipe(usb: &mut dyn Transport) -> FastbootResult<String> {
    run(usb, "erase:userdata")?;
    let _ = run(usb, "erase:metadata");
    Ok("userdata wiped".into())
}

/// Open a transport: `tcp:HOST[:PORT]` serial → TCP (default port 5554), else USB.
pub(crate) fn open(serial: Option<&str>) -> Result<Box<dyn Transport>, FastbootError> {
    if let Some(addr) = serial.and_then(|s| s.strip_prefix("tcp:")) {
        let addr = if addr.contains(':') {
            addr.to_string()
        } else {
            format!("{addr}:5554")
        };
        return crate::tcp::Tcp::connect(&addr)
            .map(|t| Box::new(t) as Box<dyn Transport>)
            .map_err(FastbootError::Usb);
    }
    match Usb::open(serial) {
        Ok(u) => Ok(Box::new(u)),
        Err(e) if e == "no fastboot device" => Err(FastbootError::NoDevice),
        Err(e) => Err(FastbootError::Usb(e)),
    }
}

/// Run a command whose device side drops the link right after `OKAY`.
pub(crate) fn run_reboot(usb: &mut dyn Transport, cmd: &str) -> FastbootResult<String> {
    match run(usb, cmd) {
        Err(FastbootError::Usb(_)) => Ok(String::new()),
        r => r,
    }
}

/// Execute `command` natively (USB or TCP).
pub fn execute(command: &FastbootCommand, serial: Option<&str>) -> Outcome {
    match open(serial) {
        Ok(mut t) => execute_on(&mut *t, command),
        Err(e) => Outcome::Fallback(e),
    }
}

/// Execute `command` over any transport (USB, TCP, or a test double).
pub fn execute_on(usb: &mut dyn Transport, command: &FastbootCommand) -> Outcome {
    let result = match command {
        FastbootCommand::OemUnlock => run(usb, "flashing unlock"),
        FastbootCommand::OemLock => run(usb, "flashing lock"),
        FastbootCommand::GetVarAll => run(usb, "getvar:all"),
        FastbootCommand::Reboot => run(usb, "reboot"),
        FastbootCommand::RebootRecovery => run(usb, "reboot-recovery"),
        FastbootCommand::RebootBootloader => run(usb, "reboot-bootloader"),
        FastbootCommand::RebootFastbootd => run(usb, "reboot-fastboot"),
        FastbootCommand::Erase { partition } => run(usb, &format!("erase:{partition}")),
        FastbootCommand::SetActiveSlot { slot } => run(usb, &format!("set_active:{slot}")),
        FastbootCommand::GetVar { name } => run(usb, &format!("getvar:{name}")),
        FastbootCommand::Raw(line) => run(usb, line),
        FastbootCommand::WipeData => wipe(usb),
        FastbootCommand::FlashPartition {
            partition,
            image_path,
        } => flash(usb, partition, image_path),
    };
    Outcome::Done(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// Scripted fake device: records writes, replays canned responses.
    struct Fake {
        sent: Vec<Vec<u8>>,
        replies: VecDeque<Vec<u8>>,
    }
    impl Fake {
        fn new(replies: &[&str]) -> Self {
            Self {
                sent: vec![],
                replies: replies.iter().map(|r| r.as_bytes().to_vec()).collect(),
            }
        }
    }
    impl Transport for Fake {
        fn send(&mut self, d: &[u8]) -> Result<(), String> {
            self.sent.push(d.to_vec());
            Ok(())
        }
        fn recv(&mut self) -> Result<Vec<u8>, String> {
            self.replies.pop_front().ok_or_else(|| "no reply".into())
        }
        fn chunk_size(&self) -> usize {
            4
        }
    }

    fn done(o: Outcome) -> FastbootResult<String> {
        match o {
            Outcome::Done(r) => r,
            Outcome::Fallback(e) => panic!("fallback: {e:?}"),
        }
    }

    #[test]
    fn getvar_collects_info() {
        let mut f = Fake::new(&["INFOproduct: x", "INFOsecure: no", "OKAY"]);
        let out = done(execute_on(&mut f, &FastbootCommand::GetVarAll)).unwrap();
        assert_eq!(out, "product: x\nsecure: no");
        assert_eq!(f.sent[0], b"getvar:all");
    }

    #[test]
    fn unlock_sends_flashing_unlock_and_surfaces_fail() {
        let mut f = Fake::new(&["FAILunlock not allowed"]);
        let r = done(execute_on(&mut f, &FastbootCommand::OemUnlock));
        assert_eq!(f.sent[0], b"flashing unlock");
        assert!(matches!(r, Err(FastbootError::CommandFailed(m)) if m.contains("not allowed")));
    }

    #[test]
    fn flash_downloads_in_chunks_then_flashes() {
        let path = std::env::temp_dir().join("dk_fb_test.img");
        std::fs::write(&path, b"0123456789").unwrap();
        let mut f = Fake::new(&["OKAY0x1000", "DATA0000000a", "OKAY", "OKAY"]);
        let cmd = FastbootCommand::FlashPartition {
            partition: "boot".into(),
            image_path: path.to_string_lossy().into_owned(),
        };
        done(execute_on(&mut f, &cmd)).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(f.sent[0], b"getvar:max-download-size");
        assert_eq!(f.sent[1], b"download:0000000a");
        assert_eq!(f.sent[2], b"0123");
        assert_eq!(f.sent[3], b"4567");
        assert_eq!(f.sent[4], b"89");
        assert_eq!(f.sent[5], b"flash:boot");
    }

    #[test]
    fn oversize_image_is_sparse_split() {
        let path = std::env::temp_dir().join("dk_fb_big.img");
        std::fs::write(&path, vec![7u8; 5 * 4096]).unwrap();
        // max-download 0x3100 -> several pieces; reply script: getvar, then per piece DATA/OKAY/OKAY.
        let mut replies = vec!["OKAY0x3100".to_string()];
        let mut f = Fake::new(&[]);
        let cmd = FastbootCommand::FlashPartition {
            partition: "system".into(),
            image_path: path.to_string_lossy().into_owned(),
        };
        // 5 blocks, 2 blocks/piece (3*4096+... limit leaves 2 blocks) -> 3 pieces.
        let mut file = std::fs::File::open(&path).unwrap();
        let pieces = crate::sparse::plan(&mut file, Some(0x3100)).unwrap();
        for p in &pieces {
            replies.push(format!("DATA{:08x}", p.len()));
            replies.push("OKAY".into());
            replies.push("OKAY".into());
        }
        f.replies = replies.into_iter().map(String::into_bytes).collect();
        done(execute_on(&mut f, &cmd)).unwrap();
        let _ = std::fs::remove_file(path);
        assert!(pieces.len() >= 2);
        let flashes = f
            .sent
            .iter()
            .filter(|s| s.as_slice() == b"flash:system")
            .count();
        assert_eq!(flashes, pieces.len());
        // first download is a sparse header
        assert_eq!(&f.sent[2][..4], &0xED26_FF3Au32.to_le_bytes());
    }

    #[test]
    fn wipe_erases_userdata_then_metadata_ignoring_fail() {
        let mut f = Fake::new(&["OKAY", "FAILno such partition"]);
        done(execute_on(&mut f, &FastbootCommand::WipeData)).unwrap();
        assert_eq!(f.sent[0], b"erase:userdata");
        assert_eq!(f.sent[1], b"erase:metadata");
    }

    #[test]
    fn simple_commands_map_to_wire() {
        let cases: Vec<(FastbootCommand, &[u8])> = vec![
            (
                FastbootCommand::Erase {
                    partition: "cache".into(),
                },
                b"erase:cache",
            ),
            (
                FastbootCommand::SetActiveSlot { slot: "b".into() },
                b"set_active:b",
            ),
            (
                FastbootCommand::GetVar {
                    name: "product".into(),
                },
                b"getvar:product",
            ),
            (
                FastbootCommand::Raw("oem device-info".into()),
                b"oem device-info",
            ),
            (FastbootCommand::RebootBootloader, b"reboot-bootloader"),
            (FastbootCommand::RebootFastbootd, b"reboot-fastboot"),
        ];
        for (c, wire) in cases {
            let mut f = Fake::new(&["OKAY"]);
            done(execute_on(&mut f, &c)).unwrap();
            assert_eq!(f.sent[0], wire);
        }
    }
}
