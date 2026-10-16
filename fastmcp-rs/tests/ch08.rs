#[test]
fn not_found_is_invalid_params() {
    assert_eq!(fastmcp_rs::not_found_code(), -32602);
}
