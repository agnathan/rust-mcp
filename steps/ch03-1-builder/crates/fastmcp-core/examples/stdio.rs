//! Serve MCP on stdin and stdout until the client closes the pipe.

use fastmcp_core::FastMcp;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Add {
    left: i64,
    right: i64,
}

#[tokio::main]
async fn main() {
    let server = FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
        Ok(Value::from(args.left + args.right))
    });
    if let Err(error) = server.run_stdio().await {
        eprintln!("stdio server stopped: {error}");
        std::process::exit(1);
    }
}
