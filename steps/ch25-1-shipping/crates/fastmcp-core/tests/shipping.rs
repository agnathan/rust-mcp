use fastmcp_core::{
    call_add, publish_plan, release_archives, Package, CRATE_LICENSE, PUBLISH_COMMAND,
};

#[test]
fn a_release_version_plans_a_dry_run() {
    let plan = publish_plan(&Package::book()).expect("plan");
    assert_eq!(plan.command, PUBLISH_COMMAND);
    assert_eq!(plan.docs_rs, "--cfg docsrs");
    assert_eq!(Package::book().license, CRATE_LICENSE);
}

#[test]
fn zero_version_is_refused() {
    let mut package = Package::book();
    package.version = "0.0.0".to_string();
    let err = publish_plan(&package).expect_err("0.0.0");
    assert_eq!(err, "refusing to publish 0.0.0");
}

#[test]
fn an_uppercase_name_is_refused() {
    let mut package = Package::book();
    package.name = "FastMcp".to_string();
    let err = publish_plan(&package).expect_err("name");
    assert_eq!(err, "crate name must be lowercase");
}

#[test]
fn three_archives_name_the_host() {
    let names = release_archives("fastmcp-core", "0.1.0");
    assert_eq!(names.len(), 3);
    assert!(names[0].ends_with("x86_64-unknown-linux-gnu.tar.gz"));
    assert!(names[2].ends_with("x86_64-pc-windows-msvc.zip"));
}

#[test]
fn add_returns_five() {
    let value = call_add(&fastmcp_core::add_server()).expect("add");
    assert_eq!(value, serde_json::json!(5));
}
