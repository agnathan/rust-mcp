//! In-memory clients, a ping, and a group of two servers.

use fastmcp_core::{infer_transport, ClientError, ClientGroup, FastMcp, McpClient, TransportKind};

#[tokio::test]
async fn ping_and_initialize_use_the_stored_handshake() {
    let client = McpClient::connect_memory(FastMcp::new("widgets"))
        .await
        .expect("connect");
    let first = client.initialize().expect("initialize");
    let second = client.initialize().expect("again");
    assert_eq!(first, second);
    assert!(client.ping().await.expect("ping"));
}

#[tokio::test]
async fn discover_has_no_initialize_result() {
    let client = McpClient::connect_discover(FastMcp::new("widgets"))
        .await
        .expect("discover");
    let error = client.initialize().expect_err("no handshake result");
    assert!(matches!(error, ClientError::NoInitializeResult));
}

#[tokio::test]
async fn a_group_prefixes_tool_names() {
    let widgets = McpClient::connect_memory(FastMcp::new("widgets"))
        .await
        .expect("widgets");
    let gadgets = McpClient::connect_memory(FastMcp::new("gadgets"))
        .await
        .expect("gadgets");
    let group = ClientGroup::new(vec![
        ("widgets".into(), widgets),
        ("gadgets".into(), gadgets),
    ])
    .expect("group");
    let names = group.list_tools().await.expect("list");
    assert_eq!(
        names,
        vec!["gadgets_status".to_string(), "widgets_status".to_string()]
    );
}

#[tokio::test]
async fn two_servers_with_one_name_collide() {
    let left = McpClient::connect_memory(FastMcp::new("left"))
        .await
        .expect("left");
    let right = McpClient::connect_memory(FastMcp::new("right"))
        .await
        .expect("right");
    let group = ClientGroup::new(vec![("one".into(), left), ("one".into(), right)]).expect("group");
    let error = group.list_tools().await.expect_err("collision");
    assert!(matches!(error, ClientError::Collision(_)));
}

#[test]
fn an_empty_group_is_rejected() {
    match ClientGroup::new(vec![]) {
        Err(ClientError::Empty) => {}
        Ok(_) => panic!("empty group was accepted"),
        Err(error) => panic!("unexpected {error}"),
    }
    assert_eq!(
        infer_transport("http://127.0.0.1/sse").expect("sse"),
        TransportKind::Sse
    );
}

#[test]
fn sse_is_recognized_and_not_a_process() {
    assert_eq!(
        infer_transport("helper.py").expect("stdio"),
        TransportKind::Stdio
    );
}
