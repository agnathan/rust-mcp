//! Bind Streamable HTTP and speak both protocol eras.
//! Does not open stdio.

use fastmcp_core::{post_mcp, FastMcp};

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"demo","version":"0.1.0"}}}"#;

const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

const DISCOVER: &str = r#"{"jsonrpc":"2.0","id":3,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"demo","version":"0.1.0"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#;

#[tokio::main]
async fn main() {
    let server = FastMcp::new("fastmcp-rs").serve_http().await.expect("bind");
    let url = server.url().to_string();
    let init = post_mcp(&url, &[("Mcp-Method", "initialize")], INIT)
        .await
        .expect("initialize");
    let session = init.header("mcp-session-id").expect("session").to_string();
    let version = init.rpc().expect("initialize json")["result"]["protocolVersion"]
        .as_str()
        .expect("version")
        .to_string();
    println!("initialize: {version}");
    println!(
        "session: {}",
        if session.is_empty() {
            "missing"
        } else {
            "present"
        }
    );
    let listed = post_mcp(
        &url,
        &[
            ("Mcp-Session-Id", &session),
            ("MCP-Protocol-Version", "2025-06-18"),
            ("Mcp-Method", "tools/list"),
        ],
        LIST,
    )
    .await
    .expect("tools/list");
    let description = listed.rpc().expect("list json")["result"]["tools"][0]["description"]
        .as_str()
        .expect("description")
        .to_string();
    println!("list: {description}");
    let discovered = post_mcp(
        &url,
        &[
            ("MCP-Protocol-Version", "2026-07-28"),
            ("Mcp-Method", "server/discover"),
        ],
        DISCOVER,
    )
    .await
    .expect("discover");
    let versions = discovered.rpc().expect("discover json")["result"]["supportedVersions"]
        .as_array()
        .expect("versions")
        .clone();
    let modern = versions.iter().any(|version| version == "2026-07-28");
    println!(
        "discover: {}",
        if modern { "2026-07-28" } else { "missing" }
    );
    server.shutdown();
}
