use std::collections::HashMap;

use fastmcp_core::{execute, Bm25, SEARCH_TOOL_NAME};

fn sample() -> Bm25 {
    Bm25::build(vec![
        ("add", "add two numbers"),
        ("weather", "forecast the sky"),
    ])
}

#[test]
fn search_ranks_the_tool_whose_text_matches() {
    let hits = sample().search("add numbers", 5);
    assert_eq!(hits, vec!["add".to_string()]);
    assert_eq!(SEARCH_TOOL_NAME, "search_tools");
}

#[test]
fn a_script_calls_two_tools_and_stops_at_the_limit() {
    let mut tools = HashMap::new();
    tools.insert("add".to_string(), 0);
    let total = execute(r#"call_tool("add", 2) + call_tool("add", 3)"#, &tools, 50).expect("run");
    assert_eq!(total, 5);
    let error = execute(r#"call_tool("add", 2) + call_tool("add", 3)"#, &tools, 1).unwrap_err();
    assert!(error.contains("tool call limit exceeded"));
}

#[test]
fn an_unknown_tool_name_is_an_error() {
    let tools = HashMap::new();
    let error = execute(r#"call_tool("missing", 1)"#, &tools, 50).unwrap_err();
    assert!(error.contains("unknown tool"));
}
