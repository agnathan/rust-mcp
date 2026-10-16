//! Three outcomes: a sum, a tool failure, and two protocol errors.
//! Does not open stdio.

use fastmcp_core::{to_mcp_error, AppError, FastMcp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
struct Add {
    left: i64,
    right: i64,
}

#[derive(Serialize, JsonSchema)]
struct Sum {
    total: i64,
}

fn main() {
    let server = FastMcp::new("fastmcp-rs").tool_app("add", "add two integers", |args: Add| {
        Ok(Sum {
            total: args.left + args.right,
        })
    });
    let value = server
        .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
        .expect("add");
    println!("ok: {value}");

    let failed = AppError::tool_failed("nope");
    println!("tool error: {failed}");

    let missing = to_mcp_error(AppError::not_found("gone"));
    println!("not found: {}", missing.code.0);

    let other = to_mcp_error(anyhow::anyhow!("boom").into());
    println!("other: {}", other.code.0);
}
