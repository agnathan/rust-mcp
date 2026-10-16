use std::borrow::Cow;

use fastmcp_core::{namespace_name, namespace_uri, rename_argument, Catalog};
use serde_json::json;

#[test]
fn a_namespace_prefixes_a_tool_and_a_resource_uri() {
    let name = namespace_name("math", "add");
    assert_eq!(name.as_ref(), "math_add");
    assert!(matches!(name, Cow::Owned(_)));

    let uri = namespace_uri("math", "file://data.txt");
    assert_eq!(uri.as_ref(), "file://math/data.txt");
}

#[test]
fn a_rename_changes_the_tool_and_one_argument() {
    let mut catalog = Catalog::new(vec![("my_tool", vec!["public"])]);
    catalog.rename_tool("my_tool", "renamed_tool");
    let args = rename_argument(&json!({"old_arg": 1, "keep": true}), "old_arg", "new_arg");
    assert_eq!(catalog.visible(), vec!["renamed_tool".to_string()]);
    assert_eq!(args, json!({"new_arg": 1, "keep": true}));
}

#[test]
fn a_later_enable_overrides_an_earlier_disable() {
    let mut catalog = Catalog::new(vec![
        ("add", vec!["public"]),
        ("secret", vec!["internal"]),
        ("safe", vec!["internal"]),
    ]);
    catalog.disable_tag("internal");
    catalog.enable_name("safe");
    assert_eq!(
        catalog.visible(),
        vec!["add".to_string(), "safe".to_string()]
    );
    assert!(catalog.list_changed());
}

#[test]
fn prompts_and_resources_show_up_as_tools() {
    let mut catalog = Catalog::new(vec![("add", Vec::<String>::new())])
        .with_prompts(vec!["greet"])
        .with_resources(vec!["file://data.txt"]);
    catalog.prompts_as_tools();
    catalog.resources_as_tools();
    assert_eq!(catalog.prompts(), &["greet".to_string()]);
    assert_eq!(catalog.resources(), &["file://data.txt".to_string()]);
    assert_eq!(
        catalog.visible(),
        vec![
            "add".to_string(),
            "list_prompts".to_string(),
            "get_prompt".to_string(),
            "list_resources".to_string(),
            "read_resource".to_string(),
        ]
    );
    assert!(catalog.list_changed());
}
