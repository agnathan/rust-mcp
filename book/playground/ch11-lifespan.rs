//! Closes run in reverse open order. No crates beyond std.

fn main() {
    let mut log = Vec::new();
    let mut hooks: Vec<Box<dyn FnOnce(&mut Vec<&str>)>> = Vec::new();
    log.push("open: db");
    hooks.push(Box::new(|log| log.push("close: db")));
    log.push("open: cache");
    hooks.push(Box::new(|log| log.push("close: cache")));
    println!("state: ready");
    while let Some(hook) = hooks.pop() {
        hook(&mut log);
    }
    for line in log {
        println!("{line}");
    }
}
