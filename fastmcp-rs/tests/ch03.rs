#[test]
fn registered_tool_is_named_add() {
    assert_eq!(fastmcp_rs::registered_tool_name(), "add");
}
