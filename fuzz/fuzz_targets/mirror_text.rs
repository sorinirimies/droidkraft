#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = droidkraft_mirror::text_command(s);
        let _ = droidkraft_mirror::parse_wm_size(s);
    }
});
