#[test]
fn cache_closes_before_the_database() {
    assert_eq!(fastmcp_rs::shutdown_order(), "cache,db");
}
