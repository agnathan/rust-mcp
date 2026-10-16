use mcp_apps::{hashed_backend_name, McpApp, UI_EXTENSION_ID};

fn main() {
    let mut app = McpApp::new("Dashboard");
    app.ui("show", "Dashboard");
    app.tool("save", "saved");
    let hashed = hashed_backend_name(app.name(), "save");
    println!("extension: {UI_EXTENSION_ID}");
    println!("model: {}", app.model_names().join(", "));
    println!("hashed: {hashed}");
    println!("body: {}", app.call_hashed(&hashed).expect("save"));
}
