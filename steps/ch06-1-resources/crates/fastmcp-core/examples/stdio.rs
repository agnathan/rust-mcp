//! Serve MCP on stdin and stdout until the client closes the pipe.

use fastmcp_core::FastMcp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
struct Add {
    left: i64,
    right: i64,
    label: Option<String>,
}

#[derive(Serialize, JsonSchema)]
struct Sum {
    total: i64,
}

#[tokio::main]
async fn main() {
    let server = FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
        let _ = args.label;
        Ok(Sum {
            total: args.left + args.right,
        })
    });
    if let Err(error) = server.run_stdio().await {
        eprintln!("stdio server stopped: {error}");
        std::process::exit(1);
    }
}
