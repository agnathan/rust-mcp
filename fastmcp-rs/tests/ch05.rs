#[test]
fn the_macro_emits_add_tool() {
    assert_eq!(fastmcp_rs::macro_tool_struct_name(), "AddTool");
}
