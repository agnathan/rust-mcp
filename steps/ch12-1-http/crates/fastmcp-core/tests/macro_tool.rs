use fastmcp_core::FastMcp;
use fastmcp_macros::tool;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct Sum {
    total: i64,
}

/// Add two integers.
#[tool]
fn add(left: i64, right: i64) -> Sum {
    Sum {
        total: left + right,
    }
}

#[tool(name = "fail", description = "always fails")]
fn fail(left: i64, right: i64) -> Result<Sum, String> {
    let _ = (left, right);
    Err("nope".to_string())
}

#[test]
fn macro_registers_add_and_uses_the_doc_comment() {
    let server = FastMcp::new("fastmcp-rs").tool_decl::<AddTool>();
    assert!(server.has_tool("add"));
    assert_eq!(
        server.tool_description("add").expect("description"),
        "Add two integers."
    );
    let value = server
        .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
        .expect("add");
    assert_eq!(value, serde_json::json!({"total": 5}));
    let direct = add(2, 3);
    assert_eq!(direct.total, 5);
}

#[test]
fn macro_result_err_is_a_tool_failure() {
    let server = FastMcp::new("fastmcp-rs").tool_decl::<FailTool>();
    let error = server
        .dispatch_tool("fail", serde_json::json!({"left": 1, "right": 2}))
        .expect_err("fails");
    assert_eq!(error, fastmcp_core::ToolError::Failed("nope".to_string()));
}
