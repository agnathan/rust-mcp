#[test]
fn the_readme_uri_is_exact() {
    assert_eq!(fastmcp_rs::static_resource_uri(), "file://readme");
}
