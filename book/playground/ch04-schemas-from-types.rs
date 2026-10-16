//! Chapter 4's schema document, without the derive macro.
//!
//! The playground ships schemars 1.2.2 without the `derive` feature, so
//! `#[derive(JsonSchema)]` does not compile there. The crate uses that derive.
//! This file is the document it prints for `Add`, and the missing-field rule
//! for `Option`.
//! https://play.rust-lang.org/

use serde_json::json;

fn main() {
    let schema = json!({
        "type": "object",
        "properties": {
            "left": {"type": "integer"},
            "right": {"type": "integer"},
            "label": {"type": ["string", "null"]}
        },
        "required": ["left", "right"]
    });
    println!("required: {}", schema["required"]);
    println!("label: {}", schema["properties"]["label"]);

    let input = json!({"left": 2, "right": 3});
    let label = input.get("label");
    println!("label omitted: {}", label.is_none());
}
