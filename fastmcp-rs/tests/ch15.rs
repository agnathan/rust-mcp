#[test]
fn the_snapshot_starts_with_status() {
    assert_eq!(fastmcp_rs::snapshot_tool_name(), "status");
}
