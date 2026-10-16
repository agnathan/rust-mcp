use fastmcp_core::{ExtensionRegistry, TaskBoard, TASKS_EXTENSION_ID};

fn board() -> TaskBoard {
    let mut registry = ExtensionRegistry::new();
    registry.insert(TASKS_EXTENSION_ID, ());
    TaskBoard::new(&registry).expect("extension")
}

#[test]
fn a_board_without_the_extension_is_refused() {
    let registry = ExtensionRegistry::new();
    let error = match TaskBoard::new(&registry) {
        Err(error) => error,
        Ok(_) => panic!("board opened without the extension"),
    };
    assert!(error.contains("tasks extension is not registered"));
    assert_eq!(TASKS_EXTENSION_ID, "io.modelcontextprotocol/tasks");
}

#[tokio::test]
async fn submit_then_poll_then_result() {
    let board = board();
    let (gate_tx, gate_rx) = std::sync::mpsc::channel::<()>();
    let id = board.submit(move || {
        gate_rx.recv().expect("gate");
        "ok".to_string()
    });
    assert_eq!(id, 1);
    assert_eq!(board.poll(id), Some("working"));
    gate_tx.send(()).expect("open");
    let value = board.wait(id).await;
    assert_eq!(value, "ok");
    assert_eq!(board.poll(id), Some("done"));
    assert_eq!(board.result(id).as_deref(), Some("ok"));
}
