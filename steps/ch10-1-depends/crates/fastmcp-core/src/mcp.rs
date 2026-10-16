//! The two MCP methods this chapter answers: `initialize` and `ping`.
//!
//! `notifications/initialized` is a notification, so it gets no response.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{ErrorCode, ProtocolError};
use crate::jsonrpc::{ErrorObject, Message, Request, Response};

/// Protocol version this step answers with. FastMCP's detached request
/// context uses the same string (`server/context.py`). Negotiation of other
/// versions is a later chapter.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

const SERVER_NAME: &str = "fastmcp-rs";
const SERVER_VERSION: &str = "0.1.0";

/// `clientInfo` / `serverInfo` on the wire. Two owned strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

/// Params of an `initialize` request. Field names on the wire are camelCase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: Value,
    pub client_info: Implementation,
}

/// Result of a successful `initialize`. `instructions` is omitted when absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: Value,
    pub server_info: Implementation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Turn one already-parsed message into the response it should produce.
///
/// A notification returns `Ok(None)`: nothing is written back. A request
/// returns `Ok(Some(response))`. [`ErrorCode::MethodNotFound`] is that
/// response, not an `Err`. `Err` means the request was `initialize` and its
/// params were not the object this chapter requires.
pub fn dispatch(message: &Message) -> Result<Option<Response>, ProtocolError> {
    match message {
        Message::Notification(_) => Ok(None),
        Message::Response(_) => Err(ProtocolError::invalid_request(
            "dispatch takes a request or a notification",
        )),
        Message::Request(request) => Ok(Some(dispatch_request(request)?)),
    }
}

/// Parse bytes, then dispatch. `?` leaves the function as soon as parse or
/// dispatch returns an error.
pub fn reply(bytes: &[u8]) -> Result<Option<Response>, ProtocolError> {
    let message = crate::jsonrpc::parse_message(bytes)?;
    dispatch(&message)
}

fn dispatch_request(request: &Request) -> Result<Response, ProtocolError> {
    match request.method.as_str() {
        "ping" => Ok(Response::success(request.id.clone(), json!({}))),
        "initialize" => initialize_response(request),
        other => Ok(Response::failure(
            Some(request.id.clone()),
            ErrorObject::from_code(
                ErrorCode::MethodNotFound,
                format!("Method not found: {other}"),
            ),
        )),
    }
}

fn initialize_response(request: &Request) -> Result<Response, ProtocolError> {
    let params = initialize_params(request)?;
    if params.protocol_version.is_empty() {
        return Err(ProtocolError::invalid_params(
            "protocolVersion must not be empty",
        ));
    }
    if params.client_info.name.is_empty() || params.client_info.version.is_empty() {
        return Err(ProtocolError::invalid_params(
            "clientInfo name and version must not be empty",
        ));
    }
    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: json!({}),
        server_info: Implementation {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
        },
        instructions: None,
    };
    let body = serde_json::to_value(&result)?;
    Ok(Response::success(request.id.clone(), body))
}

fn initialize_params(request: &Request) -> Result<InitializeParams, ProtocolError> {
    let Some(params) = request.params.clone() else {
        return Err(ProtocolError::invalid_params("initialize requires params"));
    };
    serde_json::from_value(params).map_err(|err| ProtocolError::invalid_params(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc::{parse_message, JsonRpcVersion, Notification, RequestId};

    fn init_request(id: i64) -> Request {
        Request::new(
            RequestId::Number(id),
            "initialize",
            Some(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "example", "version": "0.1.0"}
            })),
        )
    }

    #[test]
    fn ping_returns_an_empty_object() {
        let request = Request::new(RequestId::Number(7), "ping", None);
        let response = dispatch(&Message::Request(request))
            .expect("ping")
            .expect("a response");
        assert_eq!(response.id, Some(RequestId::Number(7)));
        assert_eq!(response.result, Some(json!({})));
        assert!(response.error.is_none());
        assert_eq!(response.jsonrpc, JsonRpcVersion::V2);
    }

    #[test]
    fn initialize_returns_server_info() {
        let response = dispatch(&Message::Request(init_request(1)))
            .expect("initialize")
            .expect("a response");
        let result: InitializeResult =
            serde_json::from_value(response.result.expect("result")).expect("shape");
        assert_eq!(result.protocol_version, PROTOCOL_VERSION);
        assert_eq!(result.server_info.name, "fastmcp-rs");
        assert_eq!(result.server_info.version, "0.1.0");
        assert_eq!(result.capabilities, json!({}));
        assert!(result.instructions.is_none());
        let wire = serde_json::to_value(&result).expect("wire");
        assert!(wire.get("protocolVersion").is_some());
        assert!(wire.get("serverInfo").is_some());
        assert!(wire.get("instructions").is_none());
    }

    #[test]
    fn initialized_notification_is_silent() {
        let note = Notification::new("notifications/initialized", None);
        let reply = dispatch(&Message::Notification(note)).expect("ok");
        assert!(reply.is_none());
    }

    #[test]
    fn unknown_method_is_a_response_not_a_rust_error() {
        let request = Request::new(RequestId::String("x".to_string()), "tools/list", None);
        let response = dispatch(&Message::Request(request))
            .expect("dispatch stays Ok")
            .expect("error response");
        let error = response.error.expect("error");
        assert_eq!(error.code, ErrorCode::MethodNotFound.code());
        assert!(response.result.is_none());
    }

    #[test]
    fn initialize_without_params_is_invalid_params() {
        let request = Request::new(RequestId::Number(1), "initialize", None);
        let err = dispatch(&Message::Request(request)).expect_err("params");
        assert_eq!(err.code, ErrorCode::InvalidParams);
    }

    #[test]
    fn reply_uses_question_mark_on_bad_json() {
        let err = reply(b"not-json").expect_err("parse");
        assert_eq!(err.code, ErrorCode::ParseError);
    }

    #[test]
    fn handshake_bytes_round_trip_through_reply() {
        let bytes = serde_json::to_vec(&init_request(4)).expect("bytes");
        let parsed = parse_message(&bytes).expect("parse");
        let again = reply(&bytes).expect("reply").expect("response");
        let direct = dispatch(&parsed).expect("dispatch").expect("response");
        assert_eq!(again, direct);
    }
}
