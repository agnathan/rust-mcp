//! Startup, then shutdown in reverse.
//!
//! `enter` runs now and pushes a shutdown hook. `Drop` runs the hooks from
//! last to first, and only the last clone does it.

use std::sync::{Arc, Mutex, RwLock};

/// Shared string the tools can read while the process is up.
#[derive(Clone, Default)]
pub struct AppState {
    banner: Arc<RwLock<String>>,
}

impl AppState {
    pub fn set(&self, banner: impl Into<String>) {
        *self.banner.write().expect("banner") = banner.into();
    }

    pub fn get(&self) -> String {
        self.banner.read().expect("banner").clone()
    }

    pub fn lock(&self) -> Arc<RwLock<String>> {
        Arc::clone(&self.banner)
    }
}

type Hook = Box<dyn FnOnce() + Send>;

/// One process lifetime. Clones share the log and the hooks.
#[derive(Clone)]
pub struct Lifespan {
    log: Arc<Mutex<Vec<String>>>,
    hooks: Arc<Mutex<Vec<Hook>>>,
    state: AppState,
}

impl Default for Lifespan {
    fn default() -> Self {
        Self {
            log: Arc::new(Mutex::new(Vec::new())),
            hooks: Arc::new(Mutex::new(Vec::new())),
            state: AppState::default(),
        }
    }
}

impl Lifespan {
    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn log(&self) -> Vec<String> {
        self.log.lock().expect("log").clone()
    }

    /// The log, without keeping the shutdown hooks alive.
    pub fn log_handle(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.log)
    }

    /// Open `name` now. The matching close runs on drop, after later opens.
    pub fn enter(&self, name: impl Into<String>) {
        let name = name.into();
        self.log.lock().expect("log").push(format!("open: {name}"));
        let log = Arc::clone(&self.log);
        self.hooks.lock().expect("hooks").push(Box::new(move || {
            log.lock().expect("log").push(format!("close: {name}"));
        }));
    }
}

impl Drop for Lifespan {
    fn drop(&mut self) {
        let Some(hooks) = Arc::get_mut(&mut self.hooks) else {
            return;
        };
        let mut hooks = std::mem::take(hooks.get_mut().expect("hooks"));
        while let Some(hook) = hooks.pop() {
            hook();
        }
    }
}
