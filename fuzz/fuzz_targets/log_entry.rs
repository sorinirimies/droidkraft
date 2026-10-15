#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let e = droidkraft_core::LogEntry::parse(s);
        let _ = e.message.len();
    }
});
