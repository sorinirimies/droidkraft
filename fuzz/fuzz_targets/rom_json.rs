#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = droidkraft_core::features::rom::lineage::parse_builds_json("x", s);
        let _ = droidkraft_core::features::rom::graphene::parse_metadata("x", s);
    }
});
