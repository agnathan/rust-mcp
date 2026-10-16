//! A tool waits on the desk. A second task submits the answer.

use fastmcp_core::{validate_elicitation_schema, FastMcp};
use rmcp::model::{CallToolRequestParams, ProtocolVersion};
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};

struct VersionClient;

impl ClientHandler for VersionClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = ProtocolVersion::V_2025_06_18;
        info
    }
}

#[tokio::test]
async fn the_tool_returns_the_submitted_answer() {
    let server = FastMcp::new("fastmcp-rs").elicit("confirm", "Ship it?");
    let desk = server.desk();
    tokio::spawn(async move {
        desk.submit("yes").await.expect("submit");
    });
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server.serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    let client = VersionClient.serve(client_io).await.expect("client");
    let result = client
        .call_tool(CallToolRequestParams::new("confirm"))
        .await
        .expect("confirm");
    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(serde_json::json!({"answer": "yes"}))
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn a_decline_is_a_tool_error() {
    let server = FastMcp::new("fastmcp-rs").elicit("confirm", "Ship it?");
    let desk = server.desk();
    tokio::spawn(async move {
        desk.decline().await.expect("decline");
    });
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server.serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    let client = VersionClient.serve(client_io).await.expect("client");
    let result = client
        .call_tool(CallToolRequestParams::new("confirm"))
        .await
        .expect("confirm");
    assert_eq!(result.is_error, Some(true));
    client.cancel().await.expect("cancel");
}

#[test]
fn the_flat_schema_matches_the_helper() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": { "answer": { "type": "string" } }
    });
    validate_elicitation_schema(&schema).expect("flat");
}
