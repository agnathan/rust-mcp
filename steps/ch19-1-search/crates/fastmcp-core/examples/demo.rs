use std::collections::HashMap;

use fastmcp_core::{execute, Bm25};

fn main() {
    let index = Bm25::build(vec![
        ("add", "add two numbers"),
        ("weather", "forecast the sky"),
    ]);
    let hits = index.search("add numbers", 5).join(", ");
    let mut tools = HashMap::new();
    tools.insert("add".to_string(), 0);
    let answer =
        execute(r#"call_tool("add", 2) + call_tool("add", 3)"#, &tools, 50).expect("script");
    println!("hits: {hits}");
    println!("calls: 2");
    println!("answer: {answer}");
}
