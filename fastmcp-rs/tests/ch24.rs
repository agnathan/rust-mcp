#[test]
fn settings_read_fastmcp_environment_variables() {
    assert_eq!(fastmcp_rs::settings_env_prefix(), "FASTMCP_");
}
