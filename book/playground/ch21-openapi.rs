fn operation_id(spec: &str) -> &str {
    let key = "\"operationId\":\"";
    let start = spec.find(key).expect("operationId") + key.len();
    let rest = &spec[start..];
    let end = rest.find('"').expect("end");
    &rest[..end]
}

fn main() {
    let spec = r#"{"paths":{"/widgets":{"get":{"operationId":"get_widgets"}}}}"#;
    let name = operation_id(spec);
    println!("tool: {name}");
    println!("method: GET");
    println!("body: []");
}
