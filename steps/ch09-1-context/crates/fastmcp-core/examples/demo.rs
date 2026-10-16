//! A tool logs, reports progress, and reads a resource.
//! Does not open stdio.

use fastmcp_core::FastMcp;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct Note {
    text: String,
}

fn main() {
    let server = FastMcp::new("fastmcp-rs")
        .resource("file://readme", "readme", "text/plain", "hello")
        .tool_ctx("note", "write a note", |ctx, args: Note| {
            ctx.info("working");
            ctx.report_progress(1.0, Some(2.0), Some("halfway".into()));
            let body = ctx.read_resource("file://readme").map_err(|_| "missing")?;
            let sample = ctx.sample("say hi");
            Ok(format!("{} / {body} / {sample}", args.text))
        });
    let value = server
        .dispatch_tool("note", serde_json::json!({"text": "hi"}))
        .expect("note");
    println!("note: {value}");
    println!("logs: {}", server.logs().len());
    println!("progress: {}", server.progress_notes()[0].progress);
}
