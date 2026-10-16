//! Prompts, completion, and a cursor, over the duplex pipe.

use std::collections::HashMap;

use fastmcp_core::FastMcp;
use rmcp::model::{ErrorCode, GetPromptRequestParams, PaginatedRequestParams, ProtocolVersion};
use rmcp::service::ServiceError;
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};
use serde_json::{Map, Value};

struct VersionClient;

impl ClientHandler for VersionClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = ProtocolVersion::V_2025_06_18;
        info
    }
}

fn server() -> FastMcp {
    let mut server = FastMcp::new("fastmcp-rs").with_page_size(2).prompt(
        "greet",
        "Greet someone",
        &[("name", true)],
        |args| {
            let name = args.get("name").map(String::as_str).unwrap_or("there");
            Ok(format!("Hello, {name}"))
        },
    );
    for index in 0..3 {
        let label = format!("extra-{index}");
        server = server.prompt(label, "extra", &[], |_| Ok("x".to_string()));
    }
    server.completion("greet", "name", ["Ada", "Alan", "Grace"])
}

async fn connect() -> rmcp::service::RunningService<rmcp::RoleClient, VersionClient> {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let server = server();
    tokio::spawn(async move {
        let running = server.serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    VersionClient.serve(client_io).await.expect("client")
}

#[tokio::test]
async fn a_page_of_prompts_carries_a_cursor() {
    let client = connect().await;
    let first = client
        .list_prompts(Some(PaginatedRequestParams::default()))
        .await
        .expect("first page");
    assert_eq!(first.prompts.len(), 2);
    let cursor = first.next_cursor.expect("more prompts");
    let second = client
        .list_prompts(Some(
            PaginatedRequestParams::default().with_cursor(Some(cursor)),
        ))
        .await
        .expect("second page");
    assert_eq!(second.prompts.len(), 2);
    let all = client.list_all_prompts().await.expect("all");
    let names: Vec<&str> = all.iter().map(|prompt| prompt.name.as_str()).collect();
    assert_eq!(names, ["extra-0", "extra-1", "extra-2", "greet"]);
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn get_prompt_renders_the_argument() {
    let client = connect().await;
    let mut arguments = Map::new();
    arguments.insert("name".into(), Value::from("Ada"));
    let result = client
        .get_prompt(GetPromptRequestParams::new("greet").with_arguments(arguments))
        .await
        .expect("greet");
    let text = result.messages[0].content.as_text().expect("text");
    assert_eq!(text.text, "Hello, Ada");
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn completion_filters_by_the_typed_prefix() {
    let client = connect().await;
    let values = client
        .complete_prompt_simple("greet", "name", "A")
        .await
        .expect("complete");
    assert_eq!(values, ["Ada", "Alan"]);
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn a_bad_cursor_is_invalid_params() {
    let client = connect().await;
    let error = client
        .list_prompts(Some(
            PaginatedRequestParams::default().with_cursor(Some("nope".into())),
        ))
        .await
        .expect_err("cursor");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
            assert!(data.message.contains("Invalid cursor"));
        }
        other => panic!("expected a protocol error, got {other:?}"),
    }
    client.cancel().await.expect("cancel");
}

#[test]
fn render_prompt_is_callable_without_a_session() {
    let server = server();
    let mut arguments = HashMap::new();
    arguments.insert("name".into(), "Ada".into());
    let text = server.render_prompt("greet", &arguments).expect("render");
    assert_eq!(text, "Hello, Ada");
}
