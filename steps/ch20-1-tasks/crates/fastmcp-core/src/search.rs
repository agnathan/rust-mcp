//! BM25 over tool text, and a small script that calls those tools.
//!
//! The script engine is rhai. FastMCP runs Python in a Monty sandbox. The
//! call limit is the same idea: one execution may only call tools so many times.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rhai::{Engine, EvalAltResult};

/// The tool a search transform exposes. FastMCP's default is this name.
pub const SEARCH_TOOL_NAME: &str = "search_tools";

const K1: f64 = 1.5;
const B: f64 = 0.75;

/// Tokens of two or more ASCII letters or digits, lowercased.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if current.len() >= 2 {
            tokens.push(std::mem::take(&mut current));
        } else {
            current.clear();
        }
    }
    if current.len() >= 2 {
        tokens.push(current);
    }
    tokens
}

struct Document {
    name: String,
    tf: HashMap<String, usize>,
    len: usize,
}

/// An in-memory BM25 Okapi index. `k1` is 1.5 and `b` is 0.75.
pub struct Bm25 {
    docs: Vec<Document>,
    df: HashMap<String, usize>,
    avg_dl: f64,
}

impl Bm25 {
    pub fn build(documents: Vec<(impl Into<String>, impl AsRef<str>)>) -> Self {
        let mut docs = Vec::with_capacity(documents.len());
        let mut df: HashMap<String, usize> = HashMap::new();
        let mut total_len = 0usize;
        for (name, text) in documents {
            let tokens = tokenize(text.as_ref());
            let len = tokens.len();
            total_len += len;
            let mut tf = HashMap::new();
            let mut seen = std::collections::HashSet::new();
            for token in tokens {
                *tf.entry(token.clone()).or_insert(0) += 1;
                if seen.insert(token.clone()) {
                    *df.entry(token).or_insert(0) += 1;
                }
            }
            docs.push(Document {
                name: name.into(),
                tf,
                len,
            });
        }
        let n = docs.len();
        let avg_dl = if n == 0 {
            0.0
        } else {
            total_len as f64 / n as f64
        };
        Self { docs, df, avg_dl }
    }

    /// Names of the top documents with a positive score, best first.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<String> {
        let n = self.docs.len();
        if n == 0 || top_k == 0 {
            return Vec::new();
        }
        let mut scores = vec![0.0f64; n];
        for token in tokenize(query) {
            let Some(&df) = self.df.get(&token) else {
                continue;
            };
            let idf = ((n as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();
            for (index, doc) in self.docs.iter().enumerate() {
                let tf = doc.tf.get(&token).copied().unwrap_or(0);
                if tf == 0 {
                    continue;
                }
                let tf = tf as f64;
                let dl = doc.len as f64;
                let numerator = tf * (K1 + 1.0);
                let denominator = tf + K1 * (1.0 - B + B * dl / self.avg_dl);
                scores[index] += idf * numerator / denominator;
            }
        }
        let mut ranked: Vec<usize> = (0..n).filter(|&index| scores[index] > 0.0).collect();
        ranked.sort_by(|&left, &right| {
            scores[right]
                .partial_cmp(&scores[left])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
            .into_iter()
            .take(top_k)
            .map(|index| self.docs[index].name.clone())
            .collect()
    }
}

/// Run `script` in rhai. `call_tool(name, value)` returns `value` when `name` is known.
///
/// The `(max + 1)`th call is an error. The default limit in FastMCP is 50.
pub fn execute(
    script: &str,
    tools: &HashMap<String, i64>,
    max_calls: usize,
) -> Result<i64, String> {
    let known = Arc::new(tools.clone());
    let calls = Arc::new(AtomicUsize::new(0));
    let mut engine = Engine::new();
    let known_fn = Arc::clone(&known);
    let calls_fn = Arc::clone(&calls);
    engine.register_fn(
        "call_tool",
        move |name: String, value: i64| -> Result<i64, Box<EvalAltResult>> {
            let seen = calls_fn.fetch_add(1, Ordering::SeqCst);
            if seen >= max_calls {
                return Err(format!(
                    "tool call limit exceeded: at most {max_calls} call_tool() invocations are allowed"
                )
                .into());
            }
            if !known_fn.contains_key(&name) {
                return Err(format!("unknown tool: {name}").into());
            }
            Ok(value)
        },
    );
    engine
        .eval::<i64>(script)
        .map_err(|error| error.to_string())
}

/// Sum of the values a host would pass on N separate turns.
pub fn turn_calls(tools: &HashMap<String, i64>, names: &[(&str, i64)]) -> i64 {
    names
        .iter()
        .map(
            |(name, value)| {
                if tools.contains_key(*name) {
                    *value
                } else {
                    0
                }
            },
        )
        .sum()
}
