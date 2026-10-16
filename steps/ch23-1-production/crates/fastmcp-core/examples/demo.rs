use fastmcp_core::{
    allow_call, fingerprint, highest_versions, MemoryStore, Store, Versioned, FINGERPRINT_ALGO,
};

fn main() {
    allow_call("add", |name| name == "add").expect("allow");
    let kept = highest_versions(vec![
        Versioned {
            name: "add".into(),
            version: Some("2".into()),
        },
        Versioned {
            name: "add".into(),
            version: Some("10".into()),
        },
    ]);
    let mut store = MemoryStore::new();
    store.set("banner", "ok");
    let digest = fingerprint("add", r#"{"type":"object"}"#);
    println!("allowed: add");
    println!("version: {}", kept[0].version.as_deref().unwrap_or(""));
    println!("stored: {}", store.get("banner").unwrap_or_default());
    println!("algo: {FINGERPRINT_ALGO}");
    println!("digest: {}…", &digest[..12]);
}
