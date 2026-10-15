#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut mid = s.len() / 2;
        while !s.is_char_boundary(mid) {
            mid -= 1;
        }
        let (q, t) = s.split_at(mid);
        let (q, t) = (q.to_string(), t.to_string());
        let _ = droidkraft_core::features::logcat::filter::fuzzy_match(&q, &t);
    }
});
