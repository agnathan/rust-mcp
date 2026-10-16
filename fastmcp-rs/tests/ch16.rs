#[test]
fn the_default_ping_interval_is_thirty_seconds() {
    assert_eq!(fastmcp_rs::ping_interval_ms(), "30000");
}
