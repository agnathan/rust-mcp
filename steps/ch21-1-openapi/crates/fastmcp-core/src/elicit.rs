//! A tool asks, then waits. Another task accepts, declines, or cancels.
//!
//! `Phase` is the state machine. The wait is a `oneshot`: one answer, then
//! the sender is gone. A second question opens a new channel.

use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::sync::{oneshot, Notify};

/// Where one elicitation sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Waiting,
    Accepted,
    Declined,
    Cancelled,
}

/// Why `ask` came back without a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElicitError {
    #[error("declined")]
    Declined,
    #[error("cancelled")]
    Cancelled,
}

enum Decision {
    Accepted(String),
    Declined,
    Cancelled,
}

struct Inner {
    phase: Mutex<Phase>,
    message: Mutex<String>,
    pending: Mutex<Option<oneshot::Sender<Decision>>>,
    notify: Notify,
}

/// Shared desk. Clone it into the task that will answer.
#[derive(Clone)]
pub struct Desk {
    inner: Arc<Inner>,
}

impl Default for Desk {
    fn default() -> Self {
        Self {
            inner: Arc::new(Inner {
                phase: Mutex::new(Phase::Idle),
                message: Mutex::new(String::new()),
                pending: Mutex::new(None),
                notify: Notify::new(),
            }),
        }
    }
}

impl Desk {
    pub fn phase(&self) -> Phase {
        *self.inner.phase.lock().expect("phase")
    }

    pub fn message(&self) -> String {
        self.inner.message.lock().expect("message").clone()
    }

    /// Store a waiter and block until [`Desk::submit`], [`Desk::decline`], or [`Desk::cancel`].
    ///
    /// # Errors
    ///
    /// [`ElicitError::Declined`] when the answer is a decline.
    /// [`ElicitError::Cancelled`] when the answer is a cancel, or the sender is dropped.
    pub async fn ask(&self, message: impl Into<String>) -> Result<String, ElicitError> {
        let (sender, receiver) = oneshot::channel();
        {
            *self.inner.message.lock().expect("message") = message.into();
            *self.inner.pending.lock().expect("pending") = Some(sender);
            *self.inner.phase.lock().expect("phase") = Phase::Waiting;
        }
        self.inner.notify.notify_one();
        let decision = receiver.await.map_err(|_| ElicitError::Cancelled)?;
        let mut phase = self.inner.phase.lock().expect("phase");
        match decision {
            Decision::Accepted(value) => {
                *phase = Phase::Accepted;
                Ok(value)
            }
            Decision::Declined => {
                *phase = Phase::Declined;
                Err(ElicitError::Declined)
            }
            Decision::Cancelled => {
                *phase = Phase::Cancelled;
                Err(ElicitError::Cancelled)
            }
        }
    }

    /// Accept the question that is waiting.
    ///
    /// # Errors
    ///
    /// [`ElicitError::Cancelled`] when the asker has gone away.
    pub async fn submit(&self, value: impl Into<String>) -> Result<(), ElicitError> {
        self.send(Decision::Accepted(value.into())).await
    }

    /// Decline the question that is waiting.
    ///
    /// # Errors
    ///
    /// [`ElicitError::Cancelled`] when the asker has gone away.
    pub async fn decline(&self) -> Result<(), ElicitError> {
        self.send(Decision::Declined).await
    }

    /// Cancel the question that is waiting.
    ///
    /// # Errors
    ///
    /// [`ElicitError::Cancelled`] when the asker has gone away.
    pub async fn cancel(&self) -> Result<(), ElicitError> {
        self.send(Decision::Cancelled).await
    }

    async fn send(&self, decision: Decision) -> Result<(), ElicitError> {
        let sender = self.wait_sender().await;
        sender.send(decision).map_err(|_| ElicitError::Cancelled)
    }

    async fn wait_sender(&self) -> oneshot::Sender<Decision> {
        loop {
            if let Some(sender) = self.inner.pending.lock().expect("pending").take() {
                return sender;
            }
            self.inner.notify.notified().await;
        }
    }
}

const ALLOWED_TYPES: &[&str] = &["string", "number", "integer", "boolean"];

/// A flat object of primitive fields. A nested object is rejected.
///
/// # Errors
///
/// Returns a message naming the field that broke the rule.
pub fn validate_elicitation_schema(schema: &Value) -> Result<(), String> {
    let kind = schema.get("type").and_then(Value::as_str).unwrap_or("");
    if kind != "object" {
        return Err(format!("elicitation schema must be an object, got {kind}"));
    }
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Ok(());
    };
    for (name, property) in properties {
        if property.get("const").is_some() || property.get("enum").is_some() {
            continue;
        }
        let property_type = property.get("type").and_then(Value::as_str).unwrap_or("");
        if property_type == "object" {
            return Err(format!("field {name} is a nested object"));
        }
        if property_type == "array" {
            let items_type = property
                .get("items")
                .and_then(|items| items.get("type"))
                .and_then(Value::as_str);
            if items_type == Some("object") {
                return Err(format!("field {name} is an array of objects"));
            }
            if property
                .get("items")
                .and_then(|items| items.get("enum"))
                .is_some()
            {
                continue;
            }
            return Err(format!("field {name} is an array"));
        }
        if !ALLOWED_TYPES.contains(&property_type) {
            return Err(format!("field {name} has type {property_type}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_second_round_waits_again() {
        let desk = Desk::default();
        let answerer = desk.clone();
        let first = tokio::spawn(async move { answerer.submit("yes").await });
        let value = desk.ask("Ship it?").await.expect("first");
        first.await.expect("join").expect("submit");
        assert_eq!(value, "yes");
        assert_eq!(desk.phase(), Phase::Accepted);

        let answerer = desk.clone();
        let second = tokio::spawn(async move { answerer.submit("again").await });
        let value = desk.ask("Once more?").await.expect("second");
        second.await.expect("join").expect("submit");
        assert_eq!(value, "again");
        assert_eq!(desk.message(), "Once more?");
    }

    #[test]
    fn a_nested_object_is_rejected() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "user": { "type": "object" } }
        });
        let error = validate_elicitation_schema(&schema).expect_err("nested");
        assert!(error.contains("nested object"), "{error}");
    }

    #[test]
    fn a_string_field_is_allowed() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "answer": { "type": "string" } }
        });
        validate_elicitation_schema(&schema).expect("flat");
    }
}
