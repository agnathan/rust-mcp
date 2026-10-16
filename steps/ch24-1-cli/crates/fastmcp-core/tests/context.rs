//! A tool logs, reports progress, and reads a resource through Context.
//! The schema is the argument struct only.

use fastmcp_core::FastMcp;
use rmcp::model::{CallToolRequestParams, ProtocolVersion};
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize, JsonSchema)]
struct Note {
    text: String,
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
        .resource("file://readme", "readme", "text/plain", "hello")
        .tool_ctx("note", "write a note", |ctx, args: Note| {
            ctx.info("working");
            ctx.report_progress(1.0, Some(2.0), Some("halfway".into()));
            let body = ctx.read_resource("file://readme").map_err(|_| "missing")?;
            let sample = ctx.sample("say hi");
            Ok(format!("{} / {body} / {sample}", args.text))
        })
}

#[tokio::test]
async fn the_tool_records_a_log_and_reads_the_resource() {
    let server = server();
    let running = server.clone();
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = running.serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    let client = VersionClient.serve(client_io).await.expect("client");
    let listed = client.list_tools(Default::default()).await.expect("list");
    let note = listed
        .tools
        .iter()
        .find(|tool| tool.name == "note")
        .expect("note");
    assert!(note
        .input_schema
        .get("properties")
        .unwrap()
        .get("ctx")
        .is_none());
    let mut arguments = Map::new();
    arguments.insert("text".into(), Value::from("hi"));
    let result = client
        .call_tool(CallToolRequestParams::new("note").with_arguments(arguments))
        .await
        .expect("note");
    assert_eq!(
        result.structured_content,
        Some(serde_json::json!("hi / hello / canned"))
    );
    assert_eq!(server.logs().len(), 1);
    assert_eq!(server.logs()[0].message, "working");
    assert_eq!(server.progress_notes().len(), 1);
    assert_eq!(server.progress_notes()[0].progress, 1.0);
    client.cancel().await.expect("cancel");
}
