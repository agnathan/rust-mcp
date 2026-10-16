//! Integration tests: the public crate API, one full handshake, and a
//! thousand-message parse that does not go through the benchmark harness.

use fastmcp_core::{
    dispatch, parse_message, reply, ErrorCode, InitializeResult, Message, Notification, Request,
    RequestId, PROTOCOL_VERSION,
};
use serde_json::json;

fn initialize(id: i64) -> Request {
    Request::new(
        RequestId::Number(id),
        "initialize",
        Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"roots": {"listChanged": true}},
            "clientInfo": {"name": "playground", "version": "0.1.0"}
        })),
    )
}

#[test]
fn handshake_initialize_then_notification_then_ping() {
    let init_bytes = serde_json::to_vec(&initialize(1)).expect("init bytes");
    let init_response = reply(&init_bytes).expect("init").expect("response");
    let result: InitializeResult =
        serde_json::from_value(init_response.result.expect("result")).expect("result");
    assert_eq!(result.protocol_version, PROTOCOL_VERSION);
    assert_eq!(result.server_info.name, "fastmcp-rs");

    let note = Notification::new("notifications/initialized", None);
    let note_bytes = serde_json::to_vec(&Message::Notification(note)).expect("note");
    assert!(reply(&note_bytes).expect("note reply").is_none());

    let ping = Request::new(RequestId::String("ping-1".to_string()), "ping", None);
    let ping_bytes = serde_json::to_vec(&ping).expect("ping bytes");
    let pong = reply(&ping_bytes).expect("ping").expect("pong");
    assert_eq!(pong.result, Some(json!({})));
    assert_eq!(pong.id, Some(RequestId::String("ping-1".to_string())));
}

#[test]
fn unknown_method_and_bad_initialize_params() {
    let unknown = Request::new(RequestId::Number(9), "tools/call", None);
    let response = dispatch(&Message::Request(unknown))
        .expect("ok")
        .expect("error response");
    assert_eq!(
        response.error.expect("error").code,
        ErrorCode::MethodNotFound.code()
    );

    let bad = Request::new(
        RequestId::Number(3),
        "initialize",
        Some(json!({"protocolVersion": "2025-06-18"})),
    );
    let err = reply(&serde_json::to_vec(&bad).expect("bytes")).expect_err("params");
    assert_eq!(err.code, ErrorCode::InvalidParams);
}

#[test]
fn parse_one_thousand_initialize_requests() {
    let messages: Vec<Vec<u8>> = (0..1000)
        .map(|id| serde_json::to_vec(&initialize(id)).expect("bytes"))
        .collect();
    assert_eq!(messages.len(), 1000);
    for (id, bytes) in messages.iter().enumerate() {
        match parse_message(bytes).expect("parse") {
            Message::Request(request) => {
                assert_eq!(request.method, "initialize");
                assert_eq!(request.id, RequestId::Number(id as i64));
            }
            other => panic!("expected a request, got {other:?}"),
        }
    }
}
