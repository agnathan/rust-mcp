#[test]
fn a_math_namespace_renames_add_to_math_add() {
    assert_eq!(fastmcp_rs::namespaced_tool(), "math_add");
}
