#[test]
fn the_tasks_extension_id_is_the_reverse_dns_name() {
    assert_eq!(
        fastmcp_rs::tasks_extension_id(),
        "io.modelcontextprotocol/tasks"
    );
}
