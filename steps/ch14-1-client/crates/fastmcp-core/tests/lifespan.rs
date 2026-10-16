//! Open left, then right. Drop closes right, then left.

use fastmcp_core::{FastMcp, Lifespan};

#[test]
fn shutdown_runs_in_reverse() {
    let life = Lifespan::default();
    let log = life.log_handle();
    life.enter("db");
    life.enter("cache");
    life.state().set("ready");
    assert_eq!(life.state().get(), "ready");
    drop(life);
    assert_eq!(
        log.lock().expect("log").clone(),
        vec![
            "open: db".to_string(),
            "open: cache".to_string(),
            "close: cache".to_string(),
            "close: db".to_string(),
        ]
    );
}

#[test]
fn a_cloned_server_sees_the_same_banner() {
    let server = FastMcp::new("fastmcp-rs");
    let other = server.clone();
    server.app_state().set("ready");
    assert_eq!(other.app_state().get(), "ready");
}
