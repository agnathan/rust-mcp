//! Chapter 1 in one file, for the Rust Playground.
//! Crates: serde and serde_json, both on the playground's crate list.
//! https://play.rust-lang.org/

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum RequestId {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ErrorObject {
    code: i64,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Request {
    jsonrpc: JsonRpcVersion,
    id: RequestId,
    method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Notification {
    jsonrpc: JsonRpcVersion,
    method: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Response {
    jsonrpc: JsonRpcVersion,
    id: Option<RequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<ErrorObject>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

#[derive(Debug)]
struct ProtocolError {
    code: i64,
    message: String,
}

fn parse_message(bytes: &[u8]) -> Result<Message, ProtocolError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|err| {
        if err.is_syntax() || err.is_eof() {
            ProtocolError {
                code: -32700,
                message: err.to_string(),
            }
        } else {
            ProtocolError {
                code: -32600,
                message: err.to_string(),
            }
        }
    })?;
    let object = value.as_object().ok_or_else(|| ProtocolError {
        code: -32600,
        message: "JSON-RPC message must be an object".to_string(),
    })?;
    if object.get("jsonrpc") != Some(&Value::String("2.0".to_string())) {
        return Err(ProtocolError {
            code: -32600,
            message: "jsonrpc must be the string \"2.0\"".to_string(),
        });
    }
    let has_method = object.contains_key("method");
    let has_id = object.contains_key("id");
    let has_result = object.contains_key("result");
    let has_error = object.contains_key("error");
    if has_result && has_error {
        return Err(ProtocolError {
            code: -32600,
            message: "response has both result and error".to_string(),
        });
    }
    if has_method && !has_result && !has_error {
        if has_id {
            let request: Request = serde_json::from_value(value).map_err(|err| ProtocolError {
                code: -32600,
                message: err.to_string(),
            })?;
            return Ok(Message::Request(request));
        }
        let notification: Notification =
            serde_json::from_value(value).map_err(|err| ProtocolError {
                code: -32600,
                message: err.to_string(),
            })?;
        return Ok(Message::Notification(notification));
    }
    if !has_method && has_id && (has_result || has_error) {
        let response: Response = serde_json::from_value(value).map_err(|err| ProtocolError {
            code: -32600,
            message: err.to_string(),
        })?;
        return Ok(Message::Response(response));
    }
    Err(ProtocolError {
        code: -32600,
        message: "not a request, a notification, or a response".to_string(),
    })
}

fn dispatch(message: &Message) -> Result<Option<Response>, ProtocolError> {
    match message {
        Message::Notification(_) => Ok(None),
        Message::Response(_) => Err(ProtocolError {
            code: -32600,
            message: "dispatch takes a request or a notification".to_string(),
        }),
        Message::Request(request) if request.method == "ping" => Ok(Some(Response {
            jsonrpc: JsonRpcVersion::V2,
            id: Some(request.id.clone()),
            result: Some(json!({})),
            error: None,
        })),
        Message::Request(request) if request.method == "initialize" => {
            let params = request.params.clone().ok_or_else(|| ProtocolError {
                code: -32602,
                message: "initialize requires params".to_string(),
            })?;
            let client = params
                .get("clientInfo")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .ok_or_else(|| ProtocolError {
                    code: -32602,
                    message: "clientInfo.name is required".to_string(),
                })?;
            if client.is_empty() {
                return Err(ProtocolError {
                    code: -32602,
                    message: "clientInfo name must not be empty".to_string(),
                });
            }
            Ok(Some(Response {
                jsonrpc: JsonRpcVersion::V2,
                id: Some(request.id.clone()),
                result: Some(json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "serverInfo": {"name": "fastmcp-rs", "version": "0.1.0"}
                })),
                error: None,
            }))
        }
        Message::Request(request) => Ok(Some(Response {
            jsonrpc: JsonRpcVersion::V2,
            id: Some(request.id.clone()),
            result: None,
            error: Some(ErrorObject {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        })),
    }
}

fn main() {
    let init = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"playground","version":"0.1.0"}}}"#;
    let init_message = parse_message(init).expect("parse initialize");
    let init_response = dispatch(&init_message)
        .expect("dispatch")
        .expect("response");
    let init_json = serde_json::to_string(&init_response).expect("print");
    assert!(init_json.contains("\"protocolVersion\":\"2025-06-18\""));
    assert!(init_json.contains("\"serverInfo\""));
    println!("{init_json}");

    let note = br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let note_message = parse_message(note).expect("parse notification");
    assert!(dispatch(&note_message).expect("note").is_none());
    println!("(no response)");

    let ping = br#"{"jsonrpc":"2.0","id":"ping-1","method":"ping"}"#;
    let ping_message = parse_message(ping).expect("parse ping");
    let pong = dispatch(&ping_message).expect("ping").expect("pong");
    let pong_json = serde_json::to_string(&pong).expect("print pong");
    assert_eq!(pong_json, r#"{"jsonrpc":"2.0","id":"ping-1","result":{}}"#);
    println!("{pong_json}");

    let broken = parse_message(b"{").expect_err("parse error");
    assert_eq!(broken.code, -32700);
    println!("parse error {} ({})", broken.code, broken.message);
}
