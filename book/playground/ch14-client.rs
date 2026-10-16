//! Classify a connection string. No socket, no crates beyond std.

fn infer(input: &str) -> &'static str {
    if input == "memory" {
        "memory"
    } else if input.ends_with(".py") || input.ends_with(".js") {
        "stdio"
    } else if input.contains("mcpServers") {
        "config"
    } else if input.ends_with("/sse") {
        "sse"
    } else if input.starts_with("http://") || input.starts_with("https://") {
        "http"
    } else {
        "unknown"
    }
}

fn main() {
    println!("kind: {}", infer("memory"));
    println!("http: {}", infer("http://127.0.0.1/mcp"));
    println!("sse: {}", infer("http://127.0.0.1/sse"));
    println!("tools: gadgets_status, widgets_status");
}
