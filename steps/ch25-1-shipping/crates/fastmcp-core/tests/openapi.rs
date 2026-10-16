use fastmcp_core::{call_operation, tools_from_spec, widgets_spec, MapCaller, ReqwestCaller};
use serde_json::json;

#[test]
fn an_operation_id_becomes_the_tool_name() {
    let tools = tools_from_spec(&widgets_spec()).expect("spec");
    assert_eq!(tools.base_url, "http://example.test");
    assert_eq!(tools.operations.len(), 1);
    assert_eq!(tools.operations[0].name, "get_widgets");
    assert_eq!(tools.operations[0].method, "GET");
    assert_eq!(tools.operations[0].path, "/widgets");
}

#[test]
fn a_missing_operation_id_is_a_slug() {
    let spec = json!({"paths": {"/widgets": {"get": {}}}});
    let tools = tools_from_spec(&spec).expect("spec");
    assert_eq!(tools.operations[0].name, "get_widgets");
}

#[test]
fn a_call_fills_the_path_and_uses_the_caller() {
    let spec = json!({
        "servers": [{"url": "http://example.test"}],
        "paths": {"/widgets/{id}": {"get": {"operationId": "get_widget"}}}
    });
    let tools = tools_from_spec(&spec).expect("spec");
    let caller = MapCaller::new("[]");
    let body = call_operation(&tools, "get_widget", &json!({"id": "7"}), &caller).expect("call");
    assert_eq!(body, "[]");
    assert_eq!(
        caller.last_url.lock().expect("url").as_str(),
        "http://example.test/widgets/7"
    );
    assert_eq!(caller.last_method.lock().expect("method").as_str(), "GET");
}

#[test]
fn a_reqwest_client_builds_without_sending() {
    let caller = ReqwestCaller::new().expect("client");
    let _ = caller;
}

#[test]
fn a_spec_without_paths_is_an_error() {
    let error = tools_from_spec(&json!({})).unwrap_err();
    assert!(error.contains("no paths"));
}
