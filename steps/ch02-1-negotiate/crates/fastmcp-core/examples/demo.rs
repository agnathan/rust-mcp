//! Print the version this server would agree to. Does not open stdio.
//!
//! The long-running server is `examples/stdio.rs`.

use fastmcp_core::{choose_version, era_note, era_of, SUPPORTED_VERSIONS};
use rmcp::model::ProtocolVersion;

fn line(requested: &ProtocolVersion) {
    let chosen = choose_version(requested, SUPPORTED_VERSIONS).expect("fallback exists");
    println!(
        "requested {requested} -> {chosen} ({})",
        era_note(era_of(&chosen))
    );
}

fn main() {
    line(&ProtocolVersion::V_2025_06_18);
    line(&ProtocolVersion::V_2025_11_25);
    line(&ProtocolVersion::V_2026_07_28);
    println!(
        "a discover client can still be offered {}",
        ProtocolVersion::V_2026_07_28
    );
}
