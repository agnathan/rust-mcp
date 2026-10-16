//! Register a tool, then call it through the same duplex pipe chapter 2 uses.

use fastmcp_core::FastMcp;
use rmcp::model::{CallToolRequestParams, ProtocolVersion};
use rmcp::service::ServiceError;
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize)]
struct Add {
    left: i64,
    right: i64,
}

struct VersionClient;

impl ClientHandler for VersionClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = ProtocolVersion::V_2025_06_18;
        info
    }
}

fn server() -> FastMcp {
    FastMcp::new("fastmcp-rs")
        .tool("add", "add two integers", |args: Add| {
            Ok(Value::from(args.left + args.right))
        })
        .tool("fail", "always fails", |_: Add| Err("nope".to_string()))
}

async fn connect() -> rmcp::service::RunningService<rmcp::RoleClient, VersionClient> {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server().serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    VersionClient.serve(client_io).await.expect("client")
}

#[tokio::test]
async fn list_puts_status_before_registered_tools() {
    let client = connect().await;
    let listed = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("tools/list");
    let names: Vec<&str> = listed.tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(names, ["status", "add", "fail"]);
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn call_returns_the_sum() {
    let client = connect().await;
    let mut arguments = Map::new();
    arguments.insert("left".into(), Value::from(2));
    arguments.insert("right".into(), Value::from(3));
    let result = client
        .call_tool(CallToolRequestParams::new("add").with_arguments(arguments))
        .await
        .expect("add");
    assert_eq!(result.is_error, Some(false));
    assert_eq!(result.structured_content, Some(Value::from(5)));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn unknown_name_is_invalid_params() {
    let client = connect().await;
    let error = client
        .peer()
        .call_tool_once(CallToolRequestParams::new("missing"))
        .await
        .expect_err("unknown");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code, rmcp::model::ErrorCode::INVALID_PARAMS);
            assert!(data.message.contains("unknown tool"));
        }
        other => panic!("expected a protocol error, got {other:?}"),
    }
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn tool_failure_is_a_result_the_client_can_read() {
    let client = connect().await;
    let mut arguments = Map::new();
    arguments.insert("left".into(), Value::from(1));
    arguments.insert("right".into(), Value::from(1));
    let result = client
        .call_tool(CallToolRequestParams::new("fail").with_arguments(arguments))
        .await
        .expect("tool error is still a result");
    assert_eq!(result.is_error, Some(true));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn status_still_reports_the_era() {
    let client = connect().await;
    let result = client
        .call_tool(CallToolRequestParams::new("status"))
        .await
        .expect("status");
    let text = result
        .content
        .first()
        .expect("content")
        .as_text()
        .expect("text");
    assert!(text.text.contains("handshake era"));
    client.cancel().await.expect("cancel");
}
