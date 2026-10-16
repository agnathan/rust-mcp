//! What a tool can see besides its arguments.
//!
//! `Context` is `Clone` because it holds an `Arc`. Logs and progress live in
//! `Mutex` lists so a test can read them after the call. Nothing here opens
//! a socket.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::resources::StaticResource;
use crate::tools::ToolError;

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressNote {
    pub progress: f64,
    pub total: Option<f64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogNote {
    pub level: String,
    pub message: String,
}

#[derive(Default)]
pub struct SharedNotes {
    logs: Mutex<Vec<LogNote>>,
    progress: Mutex<Vec<ProgressNote>>,
}

impl SharedNotes {
    pub fn logs(&self) -> Vec<LogNote> {
        self.logs.lock().expect("logs").clone()
    }

    pub fn progress(&self) -> Vec<ProgressNote> {
        self.progress.lock().expect("progress").clone()
    }

    pub fn clear(&self) {
        self.logs.lock().expect("logs").clear();
        self.progress.lock().expect("progress").clear();
    }
}

/// A handle passed into a tool. Cloning it shares the notes.
#[derive(Clone)]
pub struct Context {
    notes: Arc<SharedNotes>,
    resources: HashMap<String, StaticResource>,
}

impl Context {
    pub fn new(notes: Arc<SharedNotes>, resources: HashMap<String, StaticResource>) -> Self {
        Self { notes, resources }
    }

    pub fn info(&self, message: impl Into<String>) {
        self.log("info", message);
    }

    pub fn debug(&self, message: impl Into<String>) {
        self.log("debug", message);
    }

    pub fn log(&self, level: impl Into<String>, message: impl Into<String>) {
        self.notes.logs.lock().expect("logs").push(LogNote {
            level: level.into(),
            message: message.into(),
        });
    }

    pub fn report_progress(&self, progress: f64, total: Option<f64>, message: Option<String>) {
        self.notes
            .progress
            .lock()
            .expect("progress")
            .push(ProgressNote {
                progress,
                total,
                message,
            });
    }

    /// A stand-in for asking the client to sample a model.
    ///
    /// The string is fixed so a test can call this without a network.
    pub fn sample(&self, _prompt: &str) -> String {
        "canned".to_string()
    }

    pub fn clear(&self) {
        self.notes.clear();
    }

    pub fn read_resource(&self, uri: &str) -> Result<String, ToolError> {
        self.resources
            .get(uri)
            .map(|resource| resource.body.clone())
            .ok_or(ToolError::Unknown)
    }
}
