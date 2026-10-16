//! The map from a Rust error to a JSON-RPC error.
//!
//! A tool that returns `Err` is a tool result with `is_error: true`. That
//! path does not come through here. `to_mcp_error` is for a failure that
//! escaped the tool: not found, disabled, validation, an error that already
//! chose its code, or anything else.

use rmcp::model::{ErrorCode, ErrorData};

/// A failure the server can classify.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Already an MCP error. `to_mcp_error` keeps the code.
    #[error("{message}")]
    Protocol { code: ErrorCode, message: String },
    /// The named component does not exist.
    #[error("{0}")]
    NotFound(String),
    /// The named component exists and is turned off.
    #[error("{0}")]
    Disabled(String),
    /// The arguments did not validate.
    #[error("{0}")]
    Validation(String),
    /// The tool ran and reported a failure. The wire shape is `is_error`.
    #[error("{0}")]
    ToolFailed(String),
    /// Everything else. `to_mcp_error` uses `INTERNAL_ERROR`.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn disabled(message: impl Into<String>) -> Self {
        Self::Disabled(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn tool_failed(message: impl Into<String>) -> Self {
        Self::ToolFailed(message.into())
    }

    pub fn protocol(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Protocol {
            code,
            message: message.into(),
        }
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::ToolFailed(message)
    }
}

/// Translate `error` the way FastMCP's `to_mcp_error` does.
///
/// `NotFound`, `Disabled`, and `Validation` are `invalid_params`. A
/// `Protocol` value keeps its code. `ToolFailed` and `Other` are
/// `internal_error`. Callers that want a tool result should match
/// `ToolFailed` before they call this.
pub fn to_mcp_error(error: AppError) -> ErrorData {
    match error {
        AppError::Protocol { code, message } => ErrorData::new(code, message, None),
        AppError::NotFound(message)
        | AppError::Disabled(message)
        | AppError::Validation(message) => ErrorData::invalid_params(message, None),
        AppError::ToolFailed(message) => ErrorData::internal_error(message, None),
        AppError::Other(error) => ErrorData::internal_error(error.to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_is_invalid_params() {
        let data = to_mcp_error(AppError::not_found("missing"));
        assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
        assert_eq!(data.message, "missing");
    }

    #[test]
    fn a_protocol_error_keeps_its_code() {
        let data = to_mcp_error(AppError::protocol(ErrorCode::METHOD_NOT_FOUND, "nope"));
        assert_eq!(data.code, ErrorCode::METHOD_NOT_FOUND);
    }

    #[test]
    fn anyhow_is_internal() {
        let data = to_mcp_error(anyhow::anyhow!("boom").into());
        assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(data.message, "boom");
    }
}
