//! JSON-RPC 2.0 requests, notifications, and responses.
//!
//! A request has `method` and `id`. A notification has `method` and no `id`.
//! A response has `id` and exactly one of `result` or `error`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ErrorCode, ProtocolError};

/// The only version this chapter accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

/// A JSON-RPC id is a string or a number. This chapter accepts integers only.
/// `null` is not an id on a request. A response may still carry `"id": null`
/// when the request id could not be read, and that is [`Option::None`] here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

/// `error` object inside a failed response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorObject {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ErrorObject {
    pub fn from_code(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.code(),
            message: message.into(),
            data: None,
        }
    }

    pub fn from_protocol(err: &ProtocolError) -> Self {
        Self::from_code(err.code, err.message.clone())
    }
}

/// A call that expects a response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: JsonRpcVersion,
    pub id: RequestId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Request {
    pub fn new(id: RequestId, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            method: method.into(),
            params,
        }
    }
}

/// A call that expects no response. There is no `id` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Notification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            method: method.into(),
            params,
        }
    }
}

/// The reply to one request. Exactly one of `result` or `error` is set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: JsonRpcVersion,
    /// `None` serializes as `null`, which is what JSON-RPC uses when the id
    /// of a failed request could not be determined.
    pub id: Option<RequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,
}

impl Response {
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(id: Option<RequestId>, error: ErrorObject) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// One JSON-RPC message. Parsing decides the variant from which keys exist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

/// Parse one JSON-RPC message from bytes.
///
/// Invalid JSON is [`ErrorCode::ParseError`]. JSON that is not a request,
/// notification, or response is [`ErrorCode::InvalidRequest`].
pub fn parse_message(bytes: &[u8]) -> Result<Message, ProtocolError> {
    let value: Value = serde_json::from_slice(bytes)?;
    message_from_value(value)
}

fn message_from_value(value: Value) -> Result<Message, ProtocolError> {
    let Some(object) = value.as_object() else {
        return Err(ProtocolError::invalid_request(
            "JSON-RPC message must be an object",
        ));
    };

    match object.get("jsonrpc") {
        Some(version) if version == "2.0" => {}
        Some(_) => {
            return Err(ProtocolError::invalid_request(
                "jsonrpc must be the string \"2.0\"",
            ));
        }
        None => {
            return Err(ProtocolError::invalid_request("missing jsonrpc"));
        }
    }

    let has_method = object.contains_key("method");
    let has_id = object.contains_key("id");
    let has_result = object.contains_key("result");
    let has_error = object.contains_key("error");

    if has_result && has_error {
        return Err(ProtocolError::invalid_request(
            "response has both result and error",
        ));
    }

    if has_method && !has_result && !has_error {
        if has_id {
            if object.get("id").is_some_and(Value::is_null) {
                return Err(ProtocolError::invalid_request(
                    "request id must be a string or a number",
                ));
            }
            let request: Request = serde_json::from_value(value)?;
            return Ok(Message::Request(request));
        }
        let notification: Notification = serde_json::from_value(value)?;
        return Ok(Message::Notification(notification));
    }

    if !has_method && has_id && (has_result || has_error) {
        if has_result && object.get("result").is_some_and(Value::is_null) {
            return Err(ProtocolError::invalid_request(
                "this chapter does not accept a null result",
            ));
        }
        let response: Response = serde_json::from_value(value)?;
        if response.result.is_none() && response.error.is_none() {
            return Err(ProtocolError::invalid_request(
                "response needs result or error",
            ));
        }
        return Ok(Message::Response(response));
    }

    Err(ProtocolError::invalid_request(
        "not a request, a notification, or a response",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_round_trips() {
        let request = Request::new(RequestId::Number(1), "ping", None);
        let bytes = serde_json::to_vec(&request).expect("serialize");
        let parsed = parse_message(&bytes).expect("parse");
        assert_eq!(parsed, Message::Request(request));
    }

    #[test]
    fn notification_has_no_id_field() {
        let note = Notification::new("notifications/initialized", None);
        let value = serde_json::to_value(&note).expect("serialize");
        assert!(value.get("id").is_none());
        let parsed = parse_message(&serde_json::to_vec(&value).expect("bytes")).expect("parse");
        assert_eq!(parsed, Message::Notification(note));
    }

    #[test]
    fn string_id_and_error_response_round_trip() {
        let response = Response::failure(
            Some(RequestId::String("abc".to_string())),
            ErrorObject::from_code(ErrorCode::MethodNotFound, "Method not found"),
        );
        let bytes = serde_json::to_vec(&response).expect("serialize");
        let parsed = parse_message(&bytes).expect("parse");
        assert_eq!(parsed, Message::Response(response));
    }

    #[test]
    fn null_id_error_response_is_allowed() {
        let raw = br#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#;
        let parsed = parse_message(raw).expect("parse");
        match parsed {
            Message::Response(response) => {
                assert!(response.id.is_none());
                assert!(response.result.is_none());
                assert_eq!(response.error.expect("error").code, -32700);
            }
            other => panic!("expected a response, got {other:?}"),
        }
    }

    #[test]
    fn broken_json_is_a_parse_error() {
        let err = parse_message(b"{").expect_err("not json");
        assert_eq!(err.code, ErrorCode::ParseError);
    }

    #[test]
    fn wrong_version_is_an_invalid_request() {
        let err = parse_message(br#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#).expect_err("v1");
        assert_eq!(err.code, ErrorCode::InvalidRequest);
    }

    #[test]
    fn array_is_an_invalid_request() {
        let err = parse_message(b"[]").expect_err("batch");
        assert_eq!(err.code, ErrorCode::InvalidRequest);
    }

    #[test]
    fn both_result_and_error_is_rejected() {
        let raw = br#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-32603,"message":"Internal error"}}"#;
        let err = parse_message(raw).expect_err("both");
        assert_eq!(err.code, ErrorCode::InvalidRequest);
    }

    #[test]
    fn null_request_id_is_rejected() {
        let err =
            parse_message(br#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#).expect_err("null id");
        assert_eq!(err.code, ErrorCode::InvalidRequest);
    }
}
