//! The harness and the duplex agree on the tool list.

use fastmcp_core::{FastMcp, Harness, McpClient};

#[tokio::main]
async fn main() {
    let harness = Harness::new(FastMcp::new("widgets"));
    println!("memory: {}", harness.snapshot());
    let client = McpClient::connect_memory(FastMcp::new("widgets"))
        .await
        .expect("connect");
    let names = client.tool_names().await.expect("list");
    println!("duplex: {}", names.join("\n"));
}
