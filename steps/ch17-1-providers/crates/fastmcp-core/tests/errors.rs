//! A tool failure stays a result. A missing component is a protocol error.

use fastmcp_core::{AppError, FastMcp};
use rmcp::model::{CallToolRequestParams, ErrorCode, ProtocolVersion};
use rmcp::service::ServiceError;
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, JsonSchema)]
struct Add {
    left: i64,
    right: i64,
}

#[derive(Serialize, JsonSchema)]
struct Sum {
    total: i64,
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
        .tool_app("add", "add two integers", |args: Add| {
            Ok(Sum {
                total: args.left + args.right,
            })
        })
        .tool_app("fail", "tool failure", |_: Add| -> Result<Sum, AppError> {
            Err(AppError::tool_failed("nope"))
        })
        .tool_app(
            "gone",
            "missing component",
            |_: Add| -> Result<Sum, AppError> { Err(AppError::not_found("gone")) },
        )
        .tool_app("boom", "unexpected", |_: Add| -> Result<Sum, AppError> {
            Err(anyhow::anyhow!("boom").into())
        })
}

async fn connect() -> rmcp::service::RunningService<rmcp::RoleClient, VersionClient> {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server().serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    VersionClient.serve(client_io).await.expect("client")
}

fn args() -> Map<String, Value> {
    let mut arguments = Map::new();
    arguments.insert("left".into(), Value::from(2));
    arguments.insert("right".into(), Value::from(3));
    arguments
}

#[tokio::test]
async fn a_tool_failure_is_a_result() {
    let client = connect().await;
    let result = client
        .call_tool(CallToolRequestParams::new("fail").with_arguments(args()))
        .await
        .expect("tool error is a result");
    assert_eq!(result.is_error, Some(true));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn not_found_is_invalid_params_on_the_wire() {
    let client = connect().await;
    let error = client
        .call_tool(CallToolRequestParams::new("gone").with_arguments(args()))
        .await
        .expect_err("not found");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
            assert_eq!(data.message, "gone");
        }
        other => panic!("expected a protocol error, got {other:?}"),
    }
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn an_unexpected_error_is_internal() {
    let client = connect().await;
    let error = client
        .call_tool(CallToolRequestParams::new("boom").with_arguments(args()))
        .await
        .expect_err("boom");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
            assert_eq!(data.message, "boom");
        }
        other => panic!("expected a protocol error, got {other:?}"),
    }
    client.cancel().await.expect("cancel");
}
