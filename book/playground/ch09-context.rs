//! A context records progress in a shared list. No crates beyond std.

use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Context {
    notes: Arc<Mutex<Vec<f64>>>,
}

impl Context {
    fn report_progress(&self, progress: f64) {
        self.notes.lock().expect("notes").push(progress);
    }

    fn sample(&self, _prompt: &str) -> &'static str {
        "canned"
    }
}

fn main() {
    let ctx = Context {
        notes: Arc::new(Mutex::new(Vec::new())),
    };
    let other = ctx.clone();
    ctx.report_progress(1.0);
    other.report_progress(2.0);
    println!("note: hi / hello / {}", ctx.sample("say hi"));
    println!("logs: 1");
    println!("progress: {}", ctx.notes.lock().expect("notes").len());
}
