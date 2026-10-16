//! Where tools come from: a local list, an aggregate, a mount, a proxy,
//! and a directory of `*.tool` files.

use std::fs;
use std::path::Path;
use std::sync::Arc;

/// A source of tools. `call` returns `None` when this provider does not have the name.
pub trait Provider: Send + Sync {
    fn list_tools(&self) -> Vec<String>;
    fn call(&self, name: &str) -> Option<String>;
    fn instructions(&self) -> Option<String> {
        None
    }
    fn ping(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Tools registered in this process.
pub struct LocalProvider {
    tools: Vec<(String, String)>,
    instructions: Option<String>,
}

impl LocalProvider {
    pub fn new(tools: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        Self {
            tools: tools
                .into_iter()
                .map(|(name, body)| (name.into(), body.into()))
                .collect(),
            instructions: None,
        }
    }

    pub fn with_instructions(mut self, text: impl Into<String>) -> Self {
        self.instructions = Some(text.into());
        self
    }
}

impl Provider for LocalProvider {
    fn list_tools(&self) -> Vec<String> {
        self.tools.iter().map(|(name, _)| name.clone()).collect()
    }

    fn call(&self, name: &str) -> Option<String> {
        self.tools
            .iter()
            .find(|(tool, _)| tool == name)
            .map(|(_, body)| body.clone())
    }

    fn instructions(&self) -> Option<String> {
        self.instructions.clone()
    }
}

/// Several providers, queried in order. The first `call` that returns `Some` wins.
pub struct AggregateProvider {
    parts: Vec<Arc<dyn Provider>>,
}

impl AggregateProvider {
    pub fn new(parts: Vec<Arc<dyn Provider>>) -> Self {
        Self { parts }
    }
}

impl Provider for AggregateProvider {
    fn list_tools(&self) -> Vec<String> {
        let mut names = Vec::new();
        for part in &self.parts {
            names.extend(part.list_tools());
        }
        names
    }

    fn call(&self, name: &str) -> Option<String> {
        self.parts.iter().find_map(|part| part.call(name))
    }
}

/// Prefixes every tool name with `{namespace}_`.
pub struct MountedProvider {
    namespace: String,
    inner: Arc<dyn Provider>,
}

impl MountedProvider {
    pub fn new(namespace: impl Into<String>, inner: Arc<dyn Provider>) -> Self {
        Self {
            namespace: namespace.into(),
            inner,
        }
    }

    fn public_name(&self, name: &str) -> String {
        format!("{}_{name}", self.namespace)
    }
}

impl Provider for MountedProvider {
    fn list_tools(&self) -> Vec<String> {
        self.inner
            .list_tools()
            .into_iter()
            .map(|name| self.public_name(&name))
            .collect()
    }

    fn call(&self, name: &str) -> Option<String> {
        let prefix = format!("{}_", self.namespace);
        let inner = name.strip_prefix(&prefix)?;
        self.inner.call(inner)
    }
}

/// Forwards list, call, and ping. Copies upstream instructions when this proxy has none.
pub struct ProxyProvider {
    inner: Arc<dyn Provider>,
    instructions: Option<String>,
}

impl ProxyProvider {
    pub fn new(inner: Arc<dyn Provider>) -> Self {
        Self {
            inner,
            instructions: None,
        }
    }

    pub fn with_instructions(mut self, text: impl Into<String>) -> Self {
        self.instructions = Some(text.into());
        self
    }
}

impl Provider for ProxyProvider {
    fn list_tools(&self) -> Vec<String> {
        self.inner.list_tools()
    }

    fn call(&self, name: &str) -> Option<String> {
        self.inner.call(name)
    }

    fn instructions(&self) -> Option<String> {
        self.instructions
            .clone()
            .or_else(|| self.inner.instructions())
    }

    fn ping(&self) -> Result<(), String> {
        self.inner.ping()?;
        Ok(())
    }
}

/// Read `*.tool` files. The first line is the tool name. The rest is the body.
///
/// A missing directory is an empty list. Names that start with `_` are skipped.
pub fn discover_tools(root: &Path) -> Vec<(String, String)> {
    let mut paths: Vec<_> = match fs::read_dir(root) {
        Ok(entries) => entries.filter_map(|entry| entry.ok()).collect(),
        Err(_) => return Vec::new(),
    };
    paths.sort_by_key(|entry| entry.file_name());
    let mut tools = Vec::new();
    for entry in paths {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.ends_with(".tool") || file_name.starts_with('_') {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let mut lines = text.lines();
        let Some(name) = lines.next() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let body = lines.collect::<Vec<_>>().join("\n");
        tools.push((name.to_string(), body));
    }
    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_directory_is_an_empty_list() {
        let tools = discover_tools(Path::new("/no/such/fastmcp-providers-dir"));
        assert!(tools.is_empty());
    }
}
