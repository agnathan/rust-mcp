//! Hooks around a tool call: log, time, rate-limit, cache, and trim.
//!
//! A ping schedule refuses a non-positive interval and starts one task
//! per connection id.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::{FastMcp, ToolError};

const TRUNCATED: &str = "\n\n[Response truncated due to size limit]";

/// What a hook decides before the tool runs.
pub enum Before {
    Next,
    Cached(String),
    Stop(String),
}

/// One layer in the chain. `before` runs outer-first. `after` runs in reverse.
pub trait ToolHook: Send + Sync {
    fn before(&self, name: &str, args: &str) -> Before {
        let _ = (name, args);
        Before::Next
    }

    fn after(&self, name: &str, args: &str, body: &mut String) {
        let _ = (name, args, body);
    }
}

/// Names of the calls that passed this hook.
#[derive(Default)]
pub struct CallLog {
    names: Mutex<Vec<String>>,
}

impl CallLog {
    pub fn names(&self) -> Vec<String> {
        self.names.lock().expect("log").clone()
    }
}

impl ToolHook for CallLog {
    fn before(&self, name: &str, _args: &str) -> Before {
        self.names.lock().expect("log").push(name.to_string());
        Before::Next
    }
}

/// The duration of the most recent call this hook saw.
#[derive(Default)]
pub struct Timing {
    start: Mutex<Option<Instant>>,
    last: Mutex<Option<Duration>>,
}

impl Timing {
    pub fn last(&self) -> Option<Duration> {
        *self.last.lock().expect("timing")
    }
}

impl ToolHook for Timing {
    fn before(&self, _name: &str, _args: &str) -> Before {
        *self.start.lock().expect("timing") = Some(Instant::now());
        Before::Next
    }

    fn after(&self, _name: &str, _args: &str, _body: &mut String) {
        if let Some(start) = self.start.lock().expect("timing").take() {
            *self.last.lock().expect("timing") = Some(start.elapsed());
        }
    }
}

/// A token bucket. `refill_per_second` is how fast the bucket fills.
pub struct RateLimit {
    capacity: f64,
    refill_per_second: f64,
    state: Mutex<Bucket>,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

impl RateLimit {
    pub fn new(capacity: f64, refill_per_second: f64) -> Self {
        Self {
            capacity,
            refill_per_second,
            state: Mutex::new(Bucket {
                tokens: capacity,
                last: Instant::now(),
            }),
        }
    }
}

impl ToolHook for RateLimit {
    fn before(&self, _name: &str, _args: &str) -> Before {
        let mut bucket = self.state.lock().expect("bucket");
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_second).min(self.capacity);
        bucket.last = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Before::Next
        } else {
            Before::Stop("rate limit exceeded".to_string())
        }
    }
}

/// Remembers a reply so the next identical call skips the tool.
#[derive(Default)]
pub struct ResponseCache {
    entries: Mutex<HashMap<String, String>>,
}

impl ResponseCache {
    fn key(name: &str, args: &str) -> String {
        format!("{name}\n{args}")
    }
}

impl ToolHook for ResponseCache {
    fn before(&self, name: &str, args: &str) -> Before {
        match self
            .entries
            .lock()
            .expect("cache")
            .get(&Self::key(name, args))
        {
            Some(body) => Before::Cached(body.clone()),
            None => Before::Next,
        }
    }

    fn after(&self, name: &str, args: &str, body: &mut String) {
        self.entries
            .lock()
            .expect("cache")
            .insert(Self::key(name, args), body.clone());
    }
}

/// Cuts a reply that is longer than `max_bytes` and appends the suffix.
pub struct ResponseLimit {
    max_bytes: usize,
}

impl ResponseLimit {
    pub fn new(max_bytes: usize) -> Result<Self, &'static str> {
        if max_bytes == 0 {
            return Err("max_size must be positive");
        }
        Ok(Self { max_bytes })
    }
}

impl ToolHook for ResponseLimit {
    fn after(&self, _name: &str, _args: &str, body: &mut String) {
        if body.len() <= self.max_bytes {
            return;
        }
        let mut end = self.max_bytes.min(body.len());
        while !body.is_char_boundary(end) {
            end -= 1;
        }
        body.truncate(end);
        body.push_str(TRUNCATED);
    }
}

/// One ping task per connection. The interval is milliseconds and must be positive.
pub struct PingMiddleware {
    interval_ms: u64,
    sessions: Mutex<HashSet<u64>>,
}

impl PingMiddleware {
    pub fn new(interval_ms: u64) -> Result<Self, &'static str> {
        if interval_ms == 0 {
            return Err("interval_ms must be positive");
        }
        Ok(Self {
            interval_ms,
            sessions: Mutex::new(HashSet::new()),
        })
    }

    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    /// `true` when this connection id starts the one ping task.
    pub fn note_connection(&self, connection_id: u64) -> bool {
        self.sessions
            .lock()
            .expect("sessions")
            .insert(connection_id)
    }
}

impl FastMcp {
    pub fn with_hook(mut self, hook: std::sync::Arc<dyn ToolHook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Run the hook chain around `dispatch_tool`.
    pub fn invoke(&self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        let args_text = arguments.to_string();
        for hook in &self.hooks {
            match hook.before(name, &args_text) {
                Before::Next => {}
                Before::Cached(text) => return Ok(Value::String(text)),
                Before::Stop(message) => return Err(ToolError::Failed(message)),
            }
        }
        let value = self.dispatch_tool(name, arguments)?;
        if self.hooks.is_empty() {
            return Ok(value);
        }
        let original = value.to_string();
        let mut text = original.clone();
        for hook in self.hooks.iter().rev() {
            hook.after(name, &args_text, &mut text);
        }
        if text == original {
            Ok(value)
        } else {
            Ok(Value::String(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PingMiddleware;

    #[test]
    fn a_zero_interval_is_rejected() {
        assert!(PingMiddleware::new(0).is_err());
    }

    #[test]
    fn one_connection_starts_one_task() {
        let ping = PingMiddleware::new(30_000).expect("interval");
        assert!(ping.note_connection(7));
        assert!(!ping.note_connection(7));
        assert!(ping.note_connection(8));
    }
}
