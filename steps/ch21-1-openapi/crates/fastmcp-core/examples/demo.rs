use fastmcp_core::{call_operation, tools_from_spec, widgets_spec, MapCaller};

fn main() {
    let tools = tools_from_spec(&widgets_spec()).expect("spec");
    let caller = MapCaller::new("[]");
    let body =
        call_operation(&tools, "get_widgets", &serde_json::json!({}), &caller).expect("call");
    println!("tool: {}", tools.operations[0].name);
    println!("method: {}", tools.operations[0].method);
    println!("body: {body}");
}
