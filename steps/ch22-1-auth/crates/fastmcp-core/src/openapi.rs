//! Tools from an OpenAPI document, and a caller that performs the HTTP request.
//!
//! The parser reads `paths` and `operationId`. Tests use `MapCaller`. `ReqwestCaller`
//! is the client a live server would use.

use serde_json::{json, Value};

const METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    pub name: String,
    pub method: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiTools {
    pub base_url: String,
    pub operations: Vec<Operation>,
}

/// Read `paths` into tools. A missing `operationId` becomes `{method}_{slug}`.
pub fn tools_from_spec(spec: &Value) -> Result<ApiTools, String> {
    let paths = spec
        .get("paths")
        .and_then(Value::as_object)
        .ok_or("openapi spec has no paths object")?;
    let base_url = spec
        .pointer("/servers/0/url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();
    let mut operations = Vec::new();
    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };
        for method in METHODS {
            let Some(operation) = item.get(*method).and_then(Value::as_object) else {
                continue;
            };
            let name = operation
                .get("operationId")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| slug_name(method, path));
            if name.is_empty() {
                continue;
            }
            operations.push(Operation {
                name,
                method: method.to_ascii_uppercase(),
                path: path.clone(),
            });
        }
    }
    Ok(ApiTools {
        base_url,
        operations,
    })
}

fn slug_name(method: &str, path: &str) -> String {
    let mut slug = String::from(method);
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    slug.trim_matches('_').to_string()
}

pub trait HttpCaller {
    fn call(&self, method: &str, url: &str, body: Option<String>) -> Result<String, String>;
}

/// Remembers the last request and returns a fixed body.
#[derive(Debug)]
pub struct MapCaller {
    pub body: String,
    pub last_url: std::sync::Mutex<String>,
    pub last_method: std::sync::Mutex<String>,
}

impl MapCaller {
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            last_url: std::sync::Mutex::new(String::new()),
            last_method: std::sync::Mutex::new(String::new()),
        }
    }
}

impl HttpCaller for MapCaller {
    fn call(&self, method: &str, url: &str, _body: Option<String>) -> Result<String, String> {
        *self.last_method.lock().expect("method") = method.to_string();
        *self.last_url.lock().expect("url") = url.to_string();
        Ok(self.body.clone())
    }
}

/// A reqwest client. Tests do not send a request with it.
pub struct ReqwestCaller {
    client: reqwest::Client,
}

impl ReqwestCaller {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self { client })
    }

    pub async fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<String>,
    ) -> Result<String, String> {
        let method =
            reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| error.to_string())?;
        let mut request = self.client.request(method, url);
        if let Some(body) = body {
            request = request
                .header("content-type", "application/json")
                .body(body);
        }
        let response = request.send().await.map_err(|error| error.to_string())?;
        response.text().await.map_err(|error| error.to_string())
    }
}

/// Fill `{name}` from `args`, then call.
pub fn call_operation(
    tools: &ApiTools,
    name: &str,
    args: &Value,
    caller: &dyn HttpCaller,
) -> Result<String, String> {
    let operation = tools
        .operations
        .iter()
        .find(|operation| operation.name == name)
        .ok_or_else(|| format!("unknown operation: {name}"))?;
    let mut path = operation.path.clone();
    if let Some(object) = args.as_object() {
        for (key, value) in object {
            let rendered = match value {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            };
            path = path.replace(&format!("{{{key}}}"), &rendered);
        }
    }
    let url = format!("{}{path}", tools.base_url);
    let body = if operation.method == "GET" || operation.method == "DELETE" {
        None
    } else {
        Some(args.to_string())
    };
    caller.call(&operation.method, &url, body)
}

/// A small spec used by the demo and the tests.
pub fn widgets_spec() -> Value {
    json!({
        "openapi": "3.0.0",
        "servers": [{"url": "http://example.test"}],
        "paths": {
            "/widgets": {
                "get": {"operationId": "get_widgets"}
            }
        }
    })
}
