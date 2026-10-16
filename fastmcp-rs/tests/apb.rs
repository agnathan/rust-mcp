#[test]
fn the_pkce_method_is_s256() {
    assert_eq!(fastmcp_rs::pkce_method(), "S256");
}
