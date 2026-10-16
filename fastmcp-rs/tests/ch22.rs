#[test]
fn the_verifier_uses_hs256() {
    assert_eq!(fastmcp_rs::auth_algorithm(), "HS256");
}
