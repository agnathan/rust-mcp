//! Register `add` with `#[tool]` and print the sum.
//! Does not open stdio.

use fastmcp_core::FastMcp;
use fastmcp_macros::tool;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct Sum {
    total: i64,
}

/// Add two integers.
#[tool]
fn add(left: i64, right: i64) -> Sum {
    Sum {
        total: left + right,
    }
}

fn main() {
    let server = FastMcp::new("fastmcp-rs").tool_decl::<AddTool>();
    let value = server
        .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
        .expect("add");
    println!("add(2, 3) = {value}");
    println!(
        "description: {}",
        server.tool_description("add").expect("desc")
    );
    println!("direct: {}", add(2, 3).total);
}
