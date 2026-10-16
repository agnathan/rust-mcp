//! Failures that happen before a JSON-RPC response can be formed, and the
//! standard error codes a response carries when the message parsed.

use std::fmt;

/// The five JSON-RPC 2.0 error codes. The numbers are part of that spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
}

impl ErrorCode {
    /// The integer that goes in the `code` field on the wire.
    pub fn code(self) -> i64 {
        match self {
            Self::ParseError => -32700,
            Self::InvalidRequest => -32600,
            Self::MethodNotFound => -32601,
            Self::InvalidParams => -32602,
            Self::InternalError => -32603,
        }
    }

    pub fn default_message(self) -> &'static str {
        match self {
            Self::ParseError => "Parse error",
            Self::InvalidRequest => "Invalid Request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams => "Invalid params",
            Self::InternalError => "Internal error",
        }
    }
}

/// A failure of `parse_message` or `dispatch`.
///
/// A method the server does not know is not this type. That case is a normal
/// JSON-RPC response whose `error.code` is [`ErrorCode::MethodNotFound`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
}

impl ProtocolError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::ParseError, detail)
    }

    pub fn invalid_request(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRequest, detail)
    }

    pub fn invalid_params(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidParams, detail)
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code.code())
    }
}

impl std::error::Error for ProtocolError {}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        // Syntax and a truncated buffer are "the bytes were not JSON".
        // Anything else was JSON that is not the message we asked for.
        if err.is_syntax() || err.is_eof() {
            Self::parse_error(err.to_string())
        } else {
            Self::invalid_request(err.to_string())
        }
    }
}
