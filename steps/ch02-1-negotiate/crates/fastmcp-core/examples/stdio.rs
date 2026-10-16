//! Serve MCP on stdin and stdout until the client closes the pipe.

use fastmcp_core::FastMcp;

#[tokio::main]
async fn main() {
    if let Err(error) = FastMcp::new("fastmcp-rs").run_stdio().await {
        eprintln!("stdio server stopped: {error}");
        std::process::exit(1);
    }
}
