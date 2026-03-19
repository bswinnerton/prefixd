#![no_main]
use libfuzzer_sys::fuzz_target;
use prefixd::bgp::GoBgpAnnouncer;

fuzz_target!(|data: &str| {
    let announcer = GoBgpAnnouncer::new("127.0.0.1:50051".to_string());
    let _ = announcer.parse_prefix_v4(data);
    let _ = announcer.parse_prefix_v6(data);
});
