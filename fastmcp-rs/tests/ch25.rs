#[test]
fn the_publish_command_is_a_dry_run() {
    assert_eq!(fastmcp_rs::publish_command(), "cargo publish --dry-run");
}
