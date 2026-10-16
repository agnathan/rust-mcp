#[test]
fn a_mounted_status_tool_is_named_api_status() {
    assert_eq!(fastmcp_rs::mounted_tool_name(), "api_status");
}
