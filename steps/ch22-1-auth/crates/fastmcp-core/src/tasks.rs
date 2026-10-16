//! Background tasks, and a registry of extension identifiers.
//!
//! A task runs only when `io.modelcontextprotocol/tasks` is registered.
//! `submit` returns an id. `poll` reports working or done. `wait` reads the result.

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Reverse-DNS id of the tasks extension.
pub const TASKS_EXTENSION_ID: &str = "io.modelcontextprotocol/tasks";

/// Identifiers registered on the server, with an optional value.
pub struct ExtensionRegistry {
    items: HashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: impl Into<String>, value: impl Any + Send + Sync) {
        self.items.insert(id.into(), Arc::new(value));
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.contains_key(id)
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

enum State {
    Working,
    Done(String),
}

/// In-process tasks. The extension id has to be present before `new` succeeds.
pub struct TaskBoard {
    jobs: Arc<Mutex<HashMap<u64, State>>>,
    next: AtomicU64,
}

impl TaskBoard {
    pub fn new(registry: &ExtensionRegistry) -> Result<Self, String> {
        if !registry.contains(TASKS_EXTENSION_ID) {
            return Err("tasks extension is not registered".into());
        }
        Ok(Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            next: AtomicU64::new(0),
        })
    }

    /// Spawn `work` and return its id. The id starts at 1.
    pub fn submit<F>(&self, work: F) -> u64
    where
        F: FnOnce() -> String + Send + 'static,
    {
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        self.jobs.lock().expect("tasks").insert(id, State::Working);
        let jobs = Arc::clone(&self.jobs);
        tokio::spawn(async move {
            let value = work();
            let mut jobs = jobs.lock().expect("tasks");
            jobs.insert(id, State::Done(value));
        });
        id
    }

    /// `working`, `done`, or `None` when the id was never submitted.
    pub fn poll(&self, id: u64) -> Option<&'static str> {
        match self.jobs.lock().expect("tasks").get(&id) {
            Some(State::Working) => Some("working"),
            Some(State::Done(_)) => Some("done"),
            None => None,
        }
    }

    pub fn result(&self, id: u64) -> Option<String> {
        match self.jobs.lock().expect("tasks").get(&id) {
            Some(State::Done(value)) => Some(value.clone()),
            _ => None,
        }
    }

    /// Yield until the task is done, then return its text.
    pub async fn wait(&self, id: u64) -> String {
        loop {
            if let Some(value) = self.result(id) {
                return value;
            }
            tokio::task::yield_now().await;
        }
    }
}
