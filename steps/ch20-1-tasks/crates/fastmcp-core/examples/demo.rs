use fastmcp_core::{ExtensionRegistry, TaskBoard, TASKS_EXTENSION_ID};

#[tokio::main]
async fn main() {
    let mut registry = ExtensionRegistry::new();
    registry.insert(TASKS_EXTENSION_ID, ());
    let board = TaskBoard::new(&registry).expect("extension");
    let id = board.submit(|| "ok".to_string());
    let result = board.wait(id).await;
    let status = board.poll(id).unwrap_or("missing");
    println!("id: {id}");
    println!("status: {status}");
    println!("result: {result}");
}
