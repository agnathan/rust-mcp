//! Inject a database name and a token. The query is the only argument.
//! Does not open stdio.

use fastmcp_core::{Dep, FastMcp};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone)]
struct Db {
    name: String,
}

#[derive(Clone)]
struct Token(String);

#[derive(Deserialize, JsonSchema)]
struct Query {
    q: String,
}

fn main() {
    let server = FastMcp::new("fastmcp-rs")
        .provide(Db {
            name: "main".into(),
        })
        .provide(Token("secret".into()))
        .tool_inject(
            "who",
            "who is asking",
            |deps: (Dep<Db>, Dep<Token>), args: Query| {
                Ok(format!(
                    "{} {} {}",
                    deps.0.get().name,
                    deps.1.get().0,
                    args.q
                ))
            },
        );
    let value = server
        .dispatch_tool("who", serde_json::json!({"q": "widgets"}))
        .expect("who");
    let schema = server.tool_schema("who").expect("schema");
    let required = schema
        .get("required")
        .and_then(|value| value.as_array())
        .expect("required");
    println!("who: {value}");
    println!(
        "required: {}",
        required
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
