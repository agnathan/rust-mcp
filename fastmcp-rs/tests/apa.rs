#[test]
fn the_ui_extension_id_matches_fastmcp() {
    assert_eq!(fastmcp_rs::ui_extension_id(), "io.modelcontextprotocol/ui");
}
