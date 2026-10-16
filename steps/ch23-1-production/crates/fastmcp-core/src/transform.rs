//! A catalog of tools that can be prefixed, renamed, hidden, or grown.
//!
//! The server's own tool map is unchanged. This catalog is the list a client
//! would see after transforms run.

use std::borrow::Cow;
use std::collections::HashSet;

use serde_json::{Map, Value};

/// Prefix a tool or prompt name. An empty prefix borrows the original name.
pub fn namespace_name<'a>(prefix: &str, name: &'a str) -> Cow<'a, str> {
    if prefix.is_empty() {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(format!("{prefix}_{name}"))
    }
}

/// `protocol://path` becomes `protocol://prefix/path`. A URI with no scheme is borrowed.
pub fn namespace_uri<'a>(prefix: &str, uri: &'a str) -> Cow<'a, str> {
    match uri.split_once("://") {
        Some((protocol, path)) if !prefix.is_empty() => {
            Cow::Owned(format!("{protocol}://{prefix}/{path}"))
        }
        _ => Cow::Borrowed(uri),
    }
}

/// Rename one object key. Other keys stay put.
pub fn rename_argument(args: &Value, from: &str, to: &str) -> Value {
    let Value::Object(object) = args else {
        return args.clone();
    };
    let mut renamed = Map::new();
    for (key, value) in object {
        let key = if key == from {
            to.to_string()
        } else {
            key.clone()
        };
        renamed.insert(key, value.clone());
    }
    Value::Object(renamed)
}

#[derive(Clone)]
struct ToolView {
    name: String,
    tags: HashSet<String>,
    enabled: bool,
}

/// Tools, prompts, and resources after transforms.
pub struct Catalog {
    tools: Vec<ToolView>,
    prompts: Vec<String>,
    resources: Vec<String>,
    list_changed: bool,
}

impl Catalog {
    pub fn new(tools: Vec<(impl Into<String>, Vec<impl Into<String>>)>) -> Self {
        Self {
            tools: tools
                .into_iter()
                .map(|(name, tags)| ToolView {
                    name: name.into(),
                    tags: tags.into_iter().map(Into::into).collect(),
                    enabled: true,
                })
                .collect(),
            prompts: Vec::new(),
            resources: Vec::new(),
            list_changed: false,
        }
    }

    pub fn with_prompts(mut self, names: Vec<impl Into<String>>) -> Self {
        self.prompts = names.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_resources(mut self, uris: Vec<impl Into<String>>) -> Self {
        self.resources = uris.into_iter().map(Into::into).collect();
        self
    }

    pub fn list_changed(&self) -> bool {
        self.list_changed
    }

    /// Mark every tool that carries `tag` as disabled. A later enable can override it.
    pub fn disable_tag(&mut self, tag: &str) {
        for tool in &mut self.tools {
            if tool.tags.contains(tag) {
                tool.enabled = false;
            }
        }
        self.list_changed = true;
    }

    /// Mark one tool enabled, even if an earlier rule disabled it.
    pub fn enable_name(&mut self, name: &str) {
        for tool in &mut self.tools {
            if tool.name == name {
                tool.enabled = true;
            }
        }
        self.list_changed = true;
    }

    /// Rename one tool. The old name is gone from the catalog.
    pub fn rename_tool(&mut self, from: &str, to: &str) {
        for tool in &mut self.tools {
            if tool.name == from {
                tool.name = to.to_string();
            }
        }
        self.list_changed = true;
    }

    pub fn visible(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|tool| tool.enabled)
            .map(|tool| tool.name.clone())
            .collect()
    }

    /// Add `list_prompts` and `get_prompt` so a tools-only client can see prompts.
    pub fn prompts_as_tools(&mut self) {
        self.push_tool("list_prompts");
        self.push_tool("get_prompt");
        self.list_changed = true;
    }

    /// Add `list_resources` and `read_resource`.
    pub fn resources_as_tools(&mut self) {
        self.push_tool("list_resources");
        self.push_tool("read_resource");
        self.list_changed = true;
    }

    pub fn prompts(&self) -> &[String] {
        &self.prompts
    }

    pub fn resources(&self) -> &[String] {
        &self.resources
    }

    fn push_tool(&mut self, name: &str) {
        if self.tools.iter().any(|tool| tool.name == name) {
            return;
        }
        self.tools.push(ToolView {
            name: name.to_string(),
            tags: HashSet::new(),
            enabled: true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_prefix_borrows_the_name() {
        let name = namespace_name("", "add");
        assert!(matches!(name, Cow::Borrowed("add")));
    }
}
