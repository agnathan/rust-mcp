//! JSON Schema for a tool's arguments and its return value.
//!
//! `schemars` writes the schema from the type. This function does not cache.
//! A bench that calls it a thousand times is measuring generation, not a hit.

use std::sync::Arc;

use schemars::generate::SchemaSettings;
use schemars::JsonSchema;
use serde_json::{Map, Value};

/// Build a JSON Schema 2020-12 document for `T` and drop the top-level title.
pub fn json_schema_of<T: JsonSchema>() -> Arc<Map<String, Value>> {
    let generator = SchemaSettings::draft2020_12().into_generator();
    let schema = generator.into_root_schema_for::<T>();
    let Value::Object(mut object) = serde_json::to_value(schema).expect("schema is JSON") else {
        panic!("schema is a JSON object");
    };
    object.remove("title");
    object.remove("description");
    Arc::new(object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;

    #[derive(JsonSchema)]
    #[allow(dead_code)]
    struct Greet {
        name: String,
        title: Option<String>,
    }

    #[test]
    fn option_field_is_not_required() {
        let schema = json_schema_of::<Greet>();
        assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));
        let required = schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|name| name == "name"));
        assert!(required.iter().all(|name| name != "title"));
        let title = &schema["properties"]["title"];
        let encoded = serde_json::to_string(title).expect("title schema");
        assert!(
            encoded.contains("null"),
            "optional title schema should allow null, got {encoded}"
        );
    }
}
