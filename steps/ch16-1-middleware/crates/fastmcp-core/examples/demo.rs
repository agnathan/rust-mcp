//! Five hooks around one echo, and a ping interval.

use std::sync::Arc;

use fastmcp_core::{
    CallLog, FastMcp, PingMiddleware, RateLimit, ResponseCache, ResponseLimit, Timing,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
struct Echo {
    text: String,
}

fn main() {
    let server = FastMcp::new("widgets")
        .tool("echo", "echo", |args: Echo| Ok(args))
        .with_hook(Arc::new(CallLog::default()))
        .with_hook(Arc::new(Timing::default()))
        .with_hook(Arc::new(RateLimit::new(10.0, 10.0)))
        .with_hook(Arc::new(ResponseCache::default()))
        .with_hook(Arc::new(ResponseLimit::new(1_000).expect("limit")));
    let value = server
        .invoke("echo", serde_json::json!({"text": "hello"}))
        .expect("echo");
    let ping = PingMiddleware::new(30_000).expect("ping");
    println!("layers: 5");
    println!("body: {}", value["text"].as_str().unwrap_or("hello"));
    println!("ping: {}", ping.interval_ms());
}
