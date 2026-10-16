//! Read one SSE data line. No socket, no crates beyond std.
//! The listener in the chapter needs axum and rmcp.

fn first_data(body: &str) -> &str {
    body.lines()
        .find_map(|line| {
            let rest = line.strip_prefix("data:")?.trim();
            if rest.is_empty() { None } else { Some(rest) }
        })
        .expect("data line")
}

fn main() {
    let sse = "data:\n\ndata: {\"protocolVersion\":\"2025-06-18\"}\n\n";
    let payload = first_data(sse);
    println!("sse: {payload}");
    println!("session header: Mcp-Session-Id");
    println!("discover: 2026-07-28");
}
