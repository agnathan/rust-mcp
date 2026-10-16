//! List and read resources through the same duplex pipe chapter 2 uses.

use fastmcp_core::FastMcp;
use rmcp::model::{ErrorCode, ProtocolVersion, ReadResourceRequestParams, ResourceContents};
use rmcp::service::ServiceError;
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};

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
        .resource("file://license", "license", "text/plain", "mit")
        .template("res://{owner}/{repo}", "repo", "text/plain", |params| {
            let owner = param(params, "owner");
            let repo = param(params, "repo");
            Ok(format!("{owner}/{repo}"))
        })
        .template("fail://{name}", "broken", "text/plain", |_| {
            Err("cannot render".to_string())
        })
}

fn param<'a>(params: &'a [(String, String)], name: &str) -> &'a str {
    params
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
        .unwrap_or("")
}

async fn connect() -> rmcp::service::RunningService<rmcp::RoleClient, VersionClient> {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server().serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    VersionClient.serve(client_io).await.expect("client")
}

fn text_of(contents: &[ResourceContents]) -> &str {
    match contents.first().expect("contents") {
        ResourceContents::TextResourceContents { text, .. } => text,
        other => panic!("expected text, got {other:?}"),
    }
}

#[tokio::test]
async fn list_sorts_static_uris_and_advertises_the_template() {
    let client = connect().await;
    let listed = client
        .list_resources(Default::default())
        .await
        .expect("resources/list");
    let uris: Vec<&str> = listed
        .resources
        .iter()
        .map(|resource| resource.uri.as_str())
        .collect();
    assert_eq!(uris, ["file://license", "file://readme"]);
    let templates = client
        .list_resource_templates(Default::default())
        .await
        .expect("resources/templates/list");
    let patterns: Vec<&str> = templates
        .resource_templates
        .iter()
        .map(|template| template.uri_template.as_str())
        .collect();
    assert_eq!(patterns, ["fail://{name}", "res://{owner}/{repo}"]);
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn read_returns_the_static_body_and_the_rendered_template() {
    let client = connect().await;
    let readme = client
        .read_resource(ReadResourceRequestParams::new("file://readme"))
        .await
        .expect("readme");
    assert_eq!(text_of(&readme.contents), "hello");
    let repo = client
        .read_resource(ReadResourceRequestParams::new("res://acme/widgets"))
        .await
        .expect("repo");
    assert_eq!(text_of(&repo.contents), "acme/widgets");
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn unknown_uri_is_invalid_params() {
    let client = connect().await;
    let error = client
        .read_resource(ReadResourceRequestParams::new("file://missing"))
        .await
        .expect_err("missing");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
            assert!(data.message.contains("unknown resource"));
        }
        other => panic!("expected a protocol error, got {other:?}"),
    }
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn a_template_that_fails_is_an_internal_error() {
    let client = connect().await;
    let error = client
        .read_resource(ReadResourceRequestParams::new("fail://readme"))
        .await
        .expect_err("render");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
            assert!(data.message.contains("cannot render"));
        }
        other => panic!("expected a protocol error, got {other:?}"),
    }
    client.cancel().await.expect("cancel");
}
