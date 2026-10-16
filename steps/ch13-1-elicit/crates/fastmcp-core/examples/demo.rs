//! Ask once, then accept. Does not open stdio.

use fastmcp_core::{Desk, FastMcp};

#[tokio::main]
async fn main() {
    let server = FastMcp::new("fastmcp-rs").elicit("confirm", "Ship it?");
    let desk: Desk = server.desk();
    let answerer = desk.clone();
    let pending = tokio::spawn(async move {
        answerer.submit("yes").await.expect("submit");
    });
    let answer = desk.ask("Ship it?").await.expect("ask");
    pending.await.expect("join");
    println!("ask: {}", desk.message());
    println!("answer: {answer}");
    println!("phase: accepted");
}
