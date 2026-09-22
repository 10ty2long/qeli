#![no_main]
//! Exercise the shared runtime parser, document-service envelope and qeli:// decoder.
//! Imported profiles eventually reach these boundaries; malformed input must return
//! an error, never panic or loop.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let _ = qeli_core::config::parse_client_config(&text);
    let _ = qeli_core::config::editor::request(data);
    let _ = qeli_core::config::share::ClientLink::from_uri(&text);
});
