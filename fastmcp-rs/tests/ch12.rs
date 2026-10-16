#[test]
fn the_listener_serves_mcp() {
    assert_eq!(fastmcp_rs::streamable_http_path(), "/mcp");
}
