//! An in-memory harness and a setting that puts itself back.
//!
//! The harness calls the server in this process. The duplex client from
//! chapter 14 is the other way to reach the same server, through the pipe
//! that stdio tests use.

use std::sync::Mutex;

use serde_json::Value;

use crate::{FastMcp, ToolError};

static LOG_LEVEL: Mutex<String> = Mutex::new(String::new());

/// The server, held in the test, with no socket.
pub struct Harness {
    server: FastMcp,
}

impl Harness {
    pub fn new(server: FastMcp) -> Self {
        Self { server }
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.server.tool_names()
    }

    /// One line per tool, in list order. A snapshot test compares this text.
    pub fn snapshot(&self) -> String {
        self.tool_names().join("\n")
    }

    pub fn call(&self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        self.server.dispatch_tool(name, arguments)
    }
}

/// Restores the previous log level when it is dropped.
pub struct LevelGuard {
    previous: String,
}

impl Drop for LevelGuard {
    fn drop(&mut self) {
        let mut slot = LOG_LEVEL.lock().expect("log level");
        *slot = std::mem::take(&mut self.previous);
    }
}

pub fn log_level() -> String {
    let slot = LOG_LEVEL.lock().expect("log level");
    if slot.is_empty() {
        "INFO".to_string()
    } else {
        slot.clone()
    }
}

/// Set the log level until the guard is dropped.
pub fn override_log_level(level: impl Into<String>) -> LevelGuard {
    let mut slot = LOG_LEVEL.lock().expect("log level");
    let previous = if slot.is_empty() {
        "INFO".to_string()
    } else {
        slot.clone()
    };
    *slot = level.into();
    LevelGuard { previous }
}

#[cfg(test)]
mod tests {
    use super::{log_level, override_log_level};

    #[test]
    fn the_guard_puts_the_level_back() {
        assert_eq!(log_level(), "INFO");
        {
            let _guard = override_log_level("DEBUG");
            assert_eq!(log_level(), "DEBUG");
        }
        assert_eq!(log_level(), "INFO");
    }
}
