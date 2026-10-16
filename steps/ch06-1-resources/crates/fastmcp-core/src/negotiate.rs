//! Pick one MCP protocol version and name which era it belongs to.
//!
//! `rmcp` 3.5.1 already knows `2026-07-28`. Its handshake helper is
//! `pub(crate)`, so this module is the copy a reader can call and test.
//! The rule matches that helper: echo a requested handshake version the
//! server supports, and otherwise answer with the newest supported version
//! that still has `initialize`.

use rmcp::model::{ErrorData, ProtocolVersion};

/// Versions this server is willing to speak, oldest first.
///
/// A `const` slice lives in the binary. `supported_protocol_versions` borrows
/// it, so the list is not rebuilt on every request.
pub const SUPPORTED_VERSIONS: &[ProtocolVersion] = &[
    ProtocolVersion::V_2024_11_05,
    ProtocolVersion::V_2025_03_26,
    ProtocolVersion::V_2025_06_18,
    ProtocolVersion::V_2025_11_25,
    ProtocolVersion::V_2026_07_28,
];

/// The two shapes of an MCP connection this book cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolEra {
    /// `initialize`, then `notifications/initialized`. Through `2025-11-25`.
    Handshake,
    /// No `initialize`. `2026-07-28` puts the version on each request.
    Modern,
}

/// `2026-07-28` is the first revision that dropped the handshake.
pub fn era_of(version: &ProtocolVersion) -> ProtocolEra {
    if version.has_initialize() {
        ProtocolEra::Handshake
    } else {
        ProtocolEra::Modern
    }
}

/// One sentence a `tools/list` entry can show, so a test can see the branch.
pub fn era_note(era: ProtocolEra) -> &'static str {
    match era {
        ProtocolEra::Handshake => "handshake era: this session negotiated with initialize",
        ProtocolEra::Modern => {
            "2026-07-28: no initialize handshake; the version rides on each request"
        }
    }
}

/// Choose the version an `initialize` response is allowed to name.
///
/// # Errors
///
/// Returns `UNSUPPORTED_PROTOCOL_VERSION` when `supported` contains no
/// version that still has an `initialize` handshake.
pub fn choose_version(
    requested: &ProtocolVersion,
    supported: &[ProtocolVersion],
) -> Result<ProtocolVersion, ErrorData> {
    if requested.has_initialize() && supported.contains(requested) {
        return Ok(requested.clone());
    }
    supported
        .iter()
        .filter(|version| version.has_initialize())
        .max_by(|left, right| left.as_str().cmp(right.as_str()))
        .cloned()
        .ok_or_else(|| ErrorData::unsupported_protocol_version(requested.clone(), supported))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echoes_a_supported_handshake_version() {
        let chosen = choose_version(&ProtocolVersion::V_2025_06_18, SUPPORTED_VERSIONS)
            .expect("2025-06-18 is supported");
        assert_eq!(chosen, ProtocolVersion::V_2025_06_18);
        assert_eq!(era_of(&chosen), ProtocolEra::Handshake);
    }

    #[test]
    fn modern_request_on_initialize_falls_back_to_the_newest_handshake() {
        let chosen = choose_version(&ProtocolVersion::V_2026_07_28, SUPPORTED_VERSIONS)
            .expect("a handshake fallback exists");
        assert_eq!(chosen, ProtocolVersion::V_2025_11_25);
        assert_eq!(era_of(&ProtocolVersion::V_2026_07_28), ProtocolEra::Modern);
        assert!(era_note(ProtocolEra::Modern).contains("2026-07-28"));
    }

    #[test]
    fn a_list_with_no_handshake_version_is_an_error() {
        let only_modern = [ProtocolVersion::V_2026_07_28];
        let error = choose_version(&ProtocolVersion::V_2025_06_18, &only_modern)
            .expect_err("nothing to answer initialize with");
        assert_eq!(
            error.code,
            rmcp::model::ErrorCode::UNSUPPORTED_PROTOCOL_VERSION
        );
    }
}
