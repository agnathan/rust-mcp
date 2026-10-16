use mcp_apps::{
    hash_tool, hashed_backend_name, resolve_ui_mime_type, Audience, McpApp, PREFAB_RENDERER_URI,
    UI_EXTENSION_ID, UI_MIME_TYPE,
};

#[test]
fn the_model_sees_the_ui_tool_only() {
    let mut app = McpApp::new("Dashboard");
    app.ui("show", "Dashboard");
    app.tool("save", "saved");
    assert_eq!(app.model_names(), ["show"]);
    assert_eq!(app.open("show").expect("open"), "Dashboard");
    assert!(app.open("save").is_err());
}

#[test]
fn the_hashed_name_calls_the_backend_tool() {
    let hash = hash_tool("Dashboard", "save");
    assert_eq!(hash, "deb87552a0f1");
    assert_eq!(hash.len(), 12);
    let mut app = McpApp::new("Dashboard");
    app.tool("save", "saved");
    let name = hashed_backend_name("Dashboard", "save");
    assert_eq!(app.call_hashed(&name).expect("call"), "saved");
}

#[test]
fn a_ui_uri_defaults_to_the_app_mime() {
    assert_eq!(UI_EXTENSION_ID, "io.modelcontextprotocol/ui");
    assert_eq!(
        resolve_ui_mime_type(PREFAB_RENDERER_URI, None).as_deref(),
        Some(UI_MIME_TYPE)
    );
    assert_eq!(
        resolve_ui_mime_type(PREFAB_RENDERER_URI, Some("text/plain")).as_deref(),
        Some("text/plain")
    );
    assert_eq!(resolve_ui_mime_type("file://readme", None), None);
}

#[test]
fn an_app_audience_is_not_the_model() {
    assert_ne!(Audience::App, Audience::Model);
}
