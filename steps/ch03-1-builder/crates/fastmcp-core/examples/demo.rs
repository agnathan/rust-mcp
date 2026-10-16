//! Register `add` and call it. Does not open stdio.
//!
//! The long-running server is `examples/stdio.rs`.

use fastmcp_core::FastMcp;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Add {
    left: i64,
    right: i64,
}

fn main() {
    let server = FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
        Ok(Value::from(args.left + args.right))
    });
    let sum = server
        .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
        .expect("add");
    println!("add(2, 3) = {sum}");
    println!("has add: {}", server.has_tool("add"));
    println!("has status in the map: {}", server.has_tool("status"));
}
