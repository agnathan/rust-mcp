//! Chapter 3's registry, without rmcp or serde.
//! The playground does not ship rmcp. The crate deserializes JSON into P;
//! this file keeps the HashMap, the closure, and Arc<dyn Fn>.
//! https://play.rust-lang.org/

use std::collections::HashMap;
use std::sync::Arc;

struct Tool {
    call: Arc<dyn Fn(i64, i64) -> Result<i64, String> + Send + Sync>,
}

struct FastMcp {
    tools: HashMap<String, Tool>,
}

impl FastMcp {
    fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    fn tool<F>(mut self, name: &str, call: F) -> Self
    where
        F: Fn(i64, i64) -> Result<i64, String> + Send + Sync + 'static,
    {
        self.tools.insert(
            name.to_string(),
            Tool {
                call: Arc::new(call),
            },
        );
        self
    }

    fn call(&self, name: &str, left: i64, right: i64) -> Result<i64, String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("unknown tool: {name}"))?;
        (tool.call)(left, right)
    }
}

fn main() {
    let server = FastMcp::new().tool("add", |left, right| Ok(left + right));
    println!("add(2, 3) = {}", server.call("add", 2, 3).expect("add"));
    println!("{}", server.call("missing", 1, 1).expect_err("unknown"));
}
