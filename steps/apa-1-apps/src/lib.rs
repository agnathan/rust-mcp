//! An interactive app: one tool the model sees, and tools the view calls.
//!
//! The hash matches FastMCP's `hash_tool`: SHA-256 of `app_name`, a NUL, and
//! the tool name, first 12 hex characters.

use sha2::{Digest, Sha256};

/// MCP Apps extension id from `apps/config.py`.
pub const UI_EXTENSION_ID: &str = "io.modelcontextprotocol/ui";

/// MIME type for a `ui://` resource.
pub const UI_MIME_TYPE: &str = "text/html;profile=mcp-app";

/// The renderer URI FastMCP wires onto a UI entry point.
pub const PREFAB_RENDERER_URI: &str = "ui://prefab/renderer.html";

/// Hex characters kept from the SHA-256 digest.
pub const HASH_LENGTH: usize = 12;

/// Who may see a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    /// The model calls this to open the view.
    Model,
    /// The view calls this. It is not in the model list.
    App,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tool {
    name: String,
    audience: Audience,
    body: String,
}

/// A small app. It is not a Prefab renderer.
#[derive(Debug, Clone)]
pub struct McpApp {
    name: String,
    tools: Vec<Tool>,
}

impl McpApp {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tools: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Register the tool the model calls to open the view.
    pub fn ui(&mut self, name: impl Into<String>, title: impl Into<String>) {
        self.tools.push(Tool {
            name: name.into(),
            audience: Audience::Model,
            body: title.into(),
        });
    }

    /// Register a tool the view calls.
    pub fn tool(&mut self, name: impl Into<String>, body: impl Into<String>) {
        self.tools.push(Tool {
            name: name.into(),
            audience: Audience::App,
            body: body.into(),
        });
    }

    /// Names the model is allowed to see.
    pub fn model_names(&self) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|tool| tool.audience == Audience::Model)
            .map(|tool| tool.name.as_str())
            .collect()
    }

    /// Open a model-visible tool. The body is the title.
    ///
    /// # Errors
    ///
    /// The name is missing, or it is an app tool.
    pub fn open(&self, name: &str) -> Result<&str, String> {
        match self.tools.iter().find(|tool| tool.name == name) {
            Some(tool) if tool.audience == Audience::Model => Ok(tool.body.as_str()),
            Some(_) => Err(format!("not a model tool: {name}")),
            None => Err(format!("unknown tool: {name}")),
        }
    }

    /// Call `<hash>_<local_name>`.
    ///
    /// # Errors
    ///
    /// The prefix is not 12 hex characters, or no app tool has that hash and name.
    pub fn call_hashed(&self, name: &str) -> Result<&str, String> {
        let (hash, local) =
            parse_hashed_name(name).ok_or_else(|| format!("not a hashed name: {name}"))?;
        let expected = hash_tool(&self.name, local);
        if hash != expected {
            return Err(format!("unknown tool: {name}"));
        }
        self.tools
            .iter()
            .find(|tool| tool.audience == Audience::App && tool.name == local)
            .map(|tool| tool.body.as_str())
            .ok_or_else(|| format!("unknown tool: {name}"))
    }
}

/// SHA-256 of `app_name`, a NUL byte, and `tool_name`, first 12 hex characters.
pub fn hash_tool(app_name: &str, tool_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(app_name.as_bytes());
    hasher.update([0]);
    hasher.update(tool_name.as_bytes());
    let digest = hasher.finalize();
    hex_prefix(&digest, HASH_LENGTH)
}

/// `<hash>_<tool_name>`.
pub fn hashed_backend_name(app_name: &str, tool_name: &str) -> String {
    format!("{}_{tool_name}", hash_tool(app_name, tool_name))
}

/// `ui://` resources default to [`UI_MIME_TYPE`]. An explicit type wins.
pub fn resolve_ui_mime_type(uri: &str, explicit: Option<&str>) -> Option<String> {
    if let Some(explicit) = explicit {
        return Some(explicit.to_string());
    }
    if uri.to_ascii_lowercase().starts_with("ui://") {
        return Some(UI_MIME_TYPE.to_string());
    }
    None
}

fn parse_hashed_name(name: &str) -> Option<(&str, &str)> {
    let (hash, rest) = name.split_once('_')?;
    if hash.len() != HASH_LENGTH || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    if rest.is_empty() {
        return None;
    }
    Some((hash, rest))
}

fn hex_prefix(bytes: &[u8], chars: usize) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(chars);
    for byte in bytes {
        if out.len() >= chars {
            break;
        }
        out.push(HEX[(byte >> 4) as usize] as char);
        if out.len() >= chars {
            break;
        }
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Filter model-visible names out of `(name, audience)` pairs.
///
/// The benchmark builds the pairs before the timer and times this filter.
pub fn visible_model(tools: &[(String, Audience)]) -> Vec<&str> {
    tools
        .iter()
        .filter(|(_, audience)| *audience == Audience::Model)
        .map(|(name, _)| name.as_str())
        .collect()
}
