//! A call check, component versions, storage, a tracing span, and a fingerprint.
//!
//! Component versions are not protocol dates. `1` and `10` compare as numbers.
//! Redis is not in this chapter. The span is `tracing`. An OTLP exporter is not wired.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

pub const FINGERPRINT_ALGO: &str = "sha256";

/// `Ok(())` when `check` allows the tool name.
pub fn allow_call(name: &str, check: impl Fn(&str) -> bool) -> Result<(), String> {
    if check(name) {
        Ok(())
    } else {
        Err("not authorized".to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versioned {
    pub name: String,
    pub version: Option<String>,
}

/// Keep the highest version of each name. An unversioned component sorts lowest.
///
/// Numeric dotted versions compare as numbers, so `10` is newer than `2`.
/// A value that is not dotted integers compares as a string. This is component
/// versioning. It is not the protocol date from chapter 2.
pub fn highest_versions(items: Vec<Versioned>) -> Vec<Versioned> {
    let mut best: HashMap<String, Versioned> = HashMap::new();
    for item in items {
        let replace = match best.get(&item.name) {
            Some(current) => {
                version_cmp(current.version.as_deref(), item.version.as_deref())
                    == std::cmp::Ordering::Less
            }
            None => true,
        };
        if replace {
            best.insert(item.name.clone(), item);
        }
    }
    let mut kept: Vec<_> = best.into_values().collect();
    kept.sort_by(|left, right| left.name.cmp(&right.name));
    kept
}

fn version_cmp(left: Option<&str>, right: Option<&str>) -> std::cmp::Ordering {
    match (left, right) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(left), Some(right)) => compare_version_str(left, right),
    }
}

fn compare_version_str(left: &str, right: &str) -> std::cmp::Ordering {
    let left_nums = numeric_parts(left);
    let right_nums = numeric_parts(right);
    match (left_nums, right_nums) {
        (Some(left), Some(right)) => {
            let len = left.len().max(right.len());
            for index in 0..len {
                let l = left.get(index).copied().unwrap_or(0);
                let r = right.get(index).copied().unwrap_or(0);
                if l != r {
                    return l.cmp(&r);
                }
            }
            std::cmp::Ordering::Equal
        }
        _ => left.cmp(right),
    }
}

fn numeric_parts(text: &str) -> Option<Vec<u64>> {
    if text.is_empty() {
        return None;
    }
    text.split('.').map(|part| part.parse().ok()).collect()
}

pub trait Store {
    fn set(&mut self, key: &str, value: &str);
    fn get(&self, key: &str) -> Option<String>;
}

#[derive(Default)]
pub struct MemoryStore {
    map: HashMap<String, String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for MemoryStore {
    fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
    }

    fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }
}

/// One JSON object on disk. The whole map is rewritten on `set`.
pub struct FileStore {
    path: std::path::PathBuf,
    map: HashMap<String, String>,
}

impl FileStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let map = if path.exists() {
            let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            serde_json::from_str(&text).map_err(|error| error.to_string())?
        } else {
            HashMap::new()
        };
        Ok(Self { path, map })
    }

    fn flush(&self) -> Result<(), String> {
        let text = serde_json::to_string(&self.map).map_err(|error| error.to_string())?;
        fs::write(&self.path, text).map_err(|error| error.to_string())
    }
}

impl Store for FileStore {
    fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
        self.flush().expect("store");
    }

    fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }
}

/// Run `body` inside a span named `tool`.
pub fn traced_call(name: &str, body: impl FnOnce() -> String) -> String {
    let span = tracing::info_span!("tool", tool = name);
    span.in_scope(body)
}

/// SHA-256 of the name, a newline, and the schema text.
pub fn fingerprint(name: &str, schema: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b"\n");
    hasher.update(schema.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
