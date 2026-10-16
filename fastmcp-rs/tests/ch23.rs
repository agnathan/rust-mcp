#[test]
fn a_tool_fingerprint_uses_sha256() {
    assert_eq!(fastmcp_rs::fingerprint_algo(), "sha256");
}
