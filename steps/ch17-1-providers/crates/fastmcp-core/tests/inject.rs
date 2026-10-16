//! Db and Token are injected. The schema only lists the query.

use fastmcp_core::{Dep, FastMcp};
use rmcp::model::{CallToolRequestParams, ProtocolVersion};
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};

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

struct VersionClient;

impl ClientHandler for VersionClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = ProtocolVersion::V_2025_06_18;
        info
    }
}

fn server() -> FastMcp {
    FastMcp::new("fastmcp-rs")
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
        )
}

#[tokio::test]
async fn the_schema_omits_the_injected_types() {
    let server = server();
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server.serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    let client = VersionClient.serve(client_io).await.expect("client");
    let listed = client.list_tools(Default::default()).await.expect("list");
    let who = listed
        .tools
        .iter()
        .find(|tool| tool.name == "who")
        .expect("who");
    let properties = who.input_schema.get("properties").expect("properties");
    assert!(properties.get("q").is_some());
    assert!(properties.get("db").is_none());
    assert!(properties.get("token").is_none());
    let required = who
        .input_schema
        .get("required")
        .and_then(|value| value.as_array())
        .expect("required");
    assert!(required.iter().all(|name| name != "db" && name != "token"));
    let mut arguments = Map::new();
    arguments.insert("q".into(), Value::from("widgets"));
    let result = client
        .call_tool(CallToolRequestParams::new("who").with_arguments(arguments))
        .await
        .expect("who");
    assert_eq!(
        result.structured_content,
        Some(serde_json::json!("main secret widgets"))
    );
    client.cancel().await.expect("cancel");
}
