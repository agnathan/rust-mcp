#[test]
fn the_group_prefixes_the_server_name() {
    assert_eq!(fastmcp_rs::grouped_tool_name(), "widgets_status");
}
