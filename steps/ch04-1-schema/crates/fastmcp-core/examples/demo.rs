//! Register `add` and print the schema `schemars` built for `Add`.
//! Does not open stdio.
//!
//! The long-running server is `examples/stdio.rs`.

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

fn main() {
    let server = FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
        let _ = args.label;
        Ok(Sum {
            total: args.left + args.right,
        })
    });
    let schema = server
        .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
        .expect("add");
    println!("add(2, 3) = {schema}");
    let input = fastmcp_core::json_schema_of::<Add>();
    println!("required: {}", input.get("required").expect("required"));
    println!(
        "label: {}",
        input
            .get("properties")
            .and_then(|properties| properties.get("label"))
            .expect("label")
    );
}
