#[test]
fn ping_response_is_an_empty_object() {
    let json: serde_json::Value =
        serde_json::from_str(&fastmcp_rs::ping_response_json()).expect("json");
    assert_eq!(
        json,
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": {}})
    );
}
