//! A prompt is a named template. Completion is a list of strings that
//! start with what the client has typed, capped at 100.

use std::collections::HashMap;
use std::sync::Arc;

use rmcp::model::PromptArgument;

pub const MAX_COMPLETION_VALUES: usize = 100;

type RenderFn = dyn Fn(&HashMap<String, String>) -> Result<String, String> + Send + Sync;

#[derive(Clone)]
pub struct RegisteredPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    render: Arc<RenderFn>,
}

impl RegisteredPrompt {
    pub fn new<F>(
        name: impl Into<String>,
        description: impl Into<String>,
        arguments: Vec<PromptArgument>,
        render: F,
    ) -> Self
    where
        F: Fn(&HashMap<String, String>) -> Result<String, String> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            arguments,
            render: Arc::new(render),
        }
    }

    pub fn render(&self, arguments: &HashMap<String, String>) -> Result<String, String> {
        for argument in &self.arguments {
            if argument.required == Some(true) && !arguments.contains_key(&argument.name) {
                return Err(format!("missing argument: {}", argument.name));
            }
        }
        (self.render)(arguments)
    }
}

/// Keep values that start with `prefix`, then stop at 100.
///
/// The `u32` is how many values matched, before the cap. The `bool` is
/// `has_more`.
pub fn complete_values(candidates: &[String], prefix: &str) -> (Vec<String>, u32, bool) {
    let matched: Vec<String> = candidates
        .iter()
        .filter(|value| value.starts_with(prefix))
        .cloned()
        .collect();
    let total = u32::try_from(matched.len()).unwrap_or(u32::MAX);
    let has_more = matched.len() > MAX_COMPLETION_VALUES;
    let mut values = matched;
    values.truncate(MAX_COMPLETION_VALUES);
    (values, total, has_more)
}
