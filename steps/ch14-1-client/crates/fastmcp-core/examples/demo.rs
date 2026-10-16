//! Infer a transport, ping one server, and list a group of two.
//! Does not spawn a process.

use fastmcp_core::{infer_transport, ClientGroup, FastMcp, McpClient};

#[tokio::main]
async fn main() {
    let kind = infer_transport("memory").expect("memory");
    println!("kind: {kind}");
    let widgets = McpClient::connect_memory(FastMcp::new("widgets"))
        .await
        .expect("widgets");
    let gadgets = McpClient::connect_memory(FastMcp::new("gadgets"))
        .await
        .expect("gadgets");
    let ok = widgets.ping().await.expect("ping");
    println!("ping: {}", if ok { "ok" } else { "fail" });
    let group = ClientGroup::new(vec![
        ("widgets".into(), widgets),
        ("gadgets".into(), gadgets),
    ])
    .expect("group");
    let names = group.list_tools().await.expect("list");
    println!("tools: {}", names.join(", "));
}
