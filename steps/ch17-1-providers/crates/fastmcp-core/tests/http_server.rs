//! One loopback listener, two protocol eras.

use fastmcp_core::{delete_mcp, post_mcp, FastMcp, HttpServer};

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}"#;

const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

const DISCOVER: &str = r#"{"jsonrpc":"2.0","id":3,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"test","version":"0.1.0"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#;

const MODERN_LIST: &str = r#"{"jsonrpc":"2.0","id":4,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"test","version":"0.1.0"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#;

async fn spawn() -> HttpServer {
    FastMcp::new("fastmcp-rs").serve_http().await.expect("bind")
}

#[tokio::test]
async fn initialize_returns_a_session_and_lists_tools() {
    let server = spawn().await;
    let url = server.url().to_string();
    let init = post_mcp(&url, &[("Mcp-Method", "initialize")], INIT)
        .await
        .expect("initialize");
    assert_eq!(init.status, 200);
    let session = init
        .header("mcp-session-id")
        .expect("session id")
        .to_string();
    assert!(
        init.header("content-type")
            .is_some_and(|value| value.contains("text/event-stream")),
        "handshake initialize is an SSE reply"
    );
    let rpc = init.rpc().expect("initialize json");
    assert_eq!(rpc["result"]["protocolVersion"], "2025-06-18");

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
    assert_eq!(listed.status, 200);
    let rpc = listed.rpc().expect("list json");
    let description = rpc["result"]["tools"][0]["description"]
        .as_str()
        .expect("description");
    assert!(description.contains("handshake era"), "{description}");
    delete_mcp(
        &url,
        &[
            ("Mcp-Session-Id", &session),
            ("MCP-Protocol-Version", "2025-06-18"),
        ],
    )
    .await
    .expect("delete");
    server.shutdown();
}

#[tokio::test]
async fn discover_2026_has_no_session() {
    let server = spawn().await;
    let url = server.url().to_string();
    let reply = post_mcp(
        &url,
        &[
            ("MCP-Protocol-Version", "2026-07-28"),
            ("Mcp-Method", "server/discover"),
        ],
        DISCOVER,
    )
    .await
    .expect("discover");
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(reply.header("mcp-session-id").is_none());
    assert!(
        reply
            .header("content-type")
            .is_some_and(|value| value.contains("application/json")),
        "discover is a JSON reply: {:?}",
        reply.header("content-type")
    );
    let rpc = reply.rpc().expect("discover json");
    let versions = rpc["result"]["supportedVersions"]
        .as_array()
        .expect("versions");
    assert!(
        versions.iter().any(|version| version == "2026-07-28"),
        "{versions:?}"
    );
    server.shutdown();
}

#[tokio::test]
async fn modern_tools_list_names_the_version_on_the_request() {
    let server = spawn().await;
    let url = server.url().to_string();
    let listed = post_mcp(
        &url,
        &[
            ("MCP-Protocol-Version", "2026-07-28"),
            ("Mcp-Method", "tools/list"),
        ],
        MODERN_LIST,
    )
    .await
    .expect("tools/list");
    assert_eq!(listed.status, 200, "{}", listed.body);
    assert!(listed.header("mcp-session-id").is_none());
    let rpc = listed.rpc().expect("list json");
    let description = rpc["result"]["tools"][0]["description"]
        .as_str()
        .expect("description");
    assert!(description.contains("2026-07-28"), "{description}");
    server.shutdown();
}
