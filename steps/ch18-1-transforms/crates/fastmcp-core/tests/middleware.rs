//! Five hooks around one call, and a ping schedule.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use fastmcp_core::{
    CallLog, FastMcp, PingMiddleware, RateLimit, ResponseCache, ResponseLimit, ToolError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
struct Echo {
    text: String,
}

fn echo(hits: Arc<AtomicUsize>) -> FastMcp {
    FastMcp::new("widgets").tool("echo", "echo", move |args: Echo| {
        hits.fetch_add(1, Ordering::SeqCst);
        Ok(args)
    })
}

#[test]
fn five_hooks_log_time_limit_and_cache() {
    let hits = Arc::new(AtomicUsize::new(0));
    let log = Arc::new(CallLog::default());
    let server = echo(hits.clone())
        .with_hook(log.clone())
        .with_hook(Arc::new(fastmcp_core::Timing::default()))
        .with_hook(Arc::new(RateLimit::new(10.0, 0.0)))
        .with_hook(Arc::new(ResponseCache::default()))
        .with_hook(Arc::new(ResponseLimit::new(1_000).expect("limit")));
    let args = serde_json::json!({"text": "hello"});
    server.invoke("echo", args.clone()).expect("first");
    server.invoke("echo", args).expect("cached");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert_eq!(log.names(), vec!["echo".to_string(), "echo".to_string()]);
}

#[test]
fn the_bucket_stops_the_second_call() {
    let server = echo(Arc::new(AtomicUsize::new(0))).with_hook(Arc::new(RateLimit::new(1.0, 0.0)));
    let args = serde_json::json!({"text": "hello"});
    server.invoke("echo", args.clone()).expect("first");
    let error = server.invoke("echo", args).expect_err("limited");
    assert!(matches!(error, ToolError::Failed(message) if message == "rate limit exceeded"));
}

#[test]
fn a_long_reply_is_cut() {
    let server = echo(Arc::new(AtomicUsize::new(0)))
        .with_hook(Arc::new(ResponseLimit::new(8).expect("limit")));
    let value = server
        .invoke("echo", serde_json::json!({"text": "hello world"}))
        .expect("cut");
    let text = value.as_str().expect("text");
    assert!(text.contains("truncated"));
}

#[test]
fn ping_rejects_zero_and_keeps_one_task() {
    assert!(PingMiddleware::new(0).is_err());
    let ping = PingMiddleware::new(30_000).expect("ping");
    assert_eq!(ping.interval_ms(), 30_000);
    assert!(ping.note_connection(1));
    assert!(!ping.note_connection(1));
}
