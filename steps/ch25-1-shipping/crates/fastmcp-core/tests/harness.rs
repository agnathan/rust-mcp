//! The harness lists tools in process. The duplex lists them through the pipe.

use fastmcp_core::{FastMcp, Harness, McpClient};

#[test]
fn the_snapshot_is_the_tool_list() {
    let harness = Harness::new(FastMcp::new("widgets"));
    insta::assert_snapshot!(harness.snapshot(), @"status");
}

#[tokio::test]
async fn the_duplex_lists_the_same_name() {
    let harness = Harness::new(FastMcp::new("widgets"));
    let client = McpClient::connect_memory(FastMcp::new("widgets"))
        .await
        .expect("connect");
    let through_pipe = client.tool_names().await.expect("list");
    assert_eq!(harness.tool_names(), through_pipe);
}

#[test]
fn a_direct_call_reaches_the_tool() {
    let harness =
        Harness::new(
            FastMcp::new("widgets").tool("add", "add two integers", |args: Add| {
                Ok(AddOut {
                    total: args.left + args.right,
                })
            }),
        );
    let value = harness
        .call("add", serde_json::json!({"left": 2, "right": 3}))
        .expect("call");
    assert_eq!(value["total"], 5);
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Add {
    left: i64,
    right: i64,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
struct AddOut {
    total: i64,
}
