//! Chapter 2's version choice, without rmcp.
//! The playground does not ship rmcp, so the stdio server stays in the crate.
//! https://play.rust-lang.org/

const SUPPORTED: &[&str] = &[
    "2024-11-05",
    "2025-03-26",
    "2025-06-18",
    "2025-11-25",
    "2026-07-28",
];

const NO_INITIALIZE: &str = "2026-07-28";

enum ProtocolEra {
    Handshake,
    Modern,
}

fn has_initialize(version: &str) -> bool {
    version < NO_INITIALIZE
}

fn era_of(version: &str) -> ProtocolEra {
    if has_initialize(version) {
        ProtocolEra::Handshake
    } else {
        ProtocolEra::Modern
    }
}

fn era_note(era: ProtocolEra) -> &'static str {
    match era {
        ProtocolEra::Handshake => "handshake era",
        ProtocolEra::Modern => "2026-07-28",
    }
}

fn choose_version(requested: &str) -> Result<&str, &'static str> {
    if has_initialize(requested) && SUPPORTED.contains(&requested) {
        return Ok(requested);
    }
    SUPPORTED
        .iter()
        .copied()
        .filter(|version| has_initialize(version))
        .max()
        .ok_or("no handshake version")
}

fn main() {
    for requested in ["2025-06-18", "2025-11-25", "2026-07-28"] {
        let chosen = choose_version(requested).expect("fallback");
        println!(
            "requested {requested} -> {chosen} ({})",
            era_note(era_of(chosen))
        );
    }
}
