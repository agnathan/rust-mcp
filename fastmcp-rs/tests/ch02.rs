#[test]
fn negotiated_label_echoes_the_handshake_version() {
    assert_eq!(fastmcp_rs::negotiated_version_label(), "2025-06-18");
}
