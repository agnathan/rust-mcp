use std::fs;
use std::sync::Arc;

use fastmcp_core::{
    discover_tools, AggregateProvider, LocalProvider, MountedProvider, Provider, ProxyProvider,
};

fn local(name: &str, body: &str) -> Arc<dyn Provider> {
    Arc::new(LocalProvider::new(vec![(name, body)]))
}

#[test]
fn a_local_provider_lists_and_calls() {
    let provider = LocalProvider::new(vec![("status", "ok")]);
    assert_eq!(provider.list_tools(), vec!["status".to_string()]);
    assert_eq!(provider.call("status").as_deref(), Some("ok"));
    assert_eq!(provider.call("missing"), None);
}

#[test]
fn an_aggregate_asks_in_order_and_keeps_the_first_hit() {
    let first = local("status", "one");
    let second = local("status", "two");
    let extra = local("health", "up");
    let combined = AggregateProvider::new(vec![first, second, extra]);
    assert_eq!(
        combined.list_tools(),
        vec![
            "status".to_string(),
            "status".to_string(),
            "health".to_string()
        ]
    );
    assert_eq!(combined.call("status").as_deref(), Some("one"));
    assert_eq!(combined.call("health").as_deref(), Some("up"));
}

#[test]
fn a_mount_prefixes_the_name_and_strips_it_on_call() {
    let mounted = MountedProvider::new("api", local("status", "api-ok"));
    assert_eq!(mounted.list_tools(), vec!["api_status".to_string()]);
    assert_eq!(mounted.call("api_status").as_deref(), Some("api-ok"));
    assert_eq!(mounted.call("status"), None);
}

#[test]
fn a_proxy_forwards_ping_and_copies_instructions_when_it_has_none() {
    let inner = Arc::new(LocalProvider::new(vec![("status", "ok")]).with_instructions("be brief"));
    let proxy = ProxyProvider::new(inner.clone());
    assert_eq!(proxy.instructions().as_deref(), Some("be brief"));
    assert!(proxy.ping().is_ok());
    assert_eq!(proxy.call("status").as_deref(), Some("ok"));

    let own = ProxyProvider::new(inner).with_instructions("local rules");
    assert_eq!(own.instructions().as_deref(), Some("local rules"));
}

#[test]
fn discovery_reads_tool_files_and_skips_a_leading_underscore() {
    let root = std::env::temp_dir().join(format!("fastmcp-discover-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("dir");
    fs::write(root.join("b.tool"), "beta\nsecond\n").expect("b");
    fs::write(root.join("a.tool"), "alpha\nfirst\n").expect("a");
    fs::write(root.join("_hidden.tool"), "secret\nnope\n").expect("hidden");
    fs::write(root.join("notes.txt"), "not a tool\n").expect("txt");

    let tools = discover_tools(&root);
    assert_eq!(
        tools,
        vec![
            ("alpha".to_string(), "first".to_string()),
            ("beta".to_string(), "second".to_string()),
        ]
    );
    let _ = fs::remove_dir_all(&root);
}
