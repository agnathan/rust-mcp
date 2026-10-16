use fastmcp_core::{
    allow_call, fingerprint, highest_versions, traced_call, FileStore, MemoryStore, Store,
    Versioned, FINGERPRINT_ALGO,
};

#[test]
fn a_check_can_refuse_the_call() {
    assert!(allow_call("add", |name| name == "add").is_ok());
    let error = allow_call("secret", |_| false).unwrap_err();
    assert_eq!(error, "not authorized");
}

#[test]
fn the_highest_component_version_wins_and_ten_beats_two() {
    let kept = highest_versions(vec![
        Versioned {
            name: "add".into(),
            version: Some("2".into()),
        },
        Versioned {
            name: "add".into(),
            version: Some("10".into()),
        },
        Versioned {
            name: "add".into(),
            version: None,
        },
        Versioned {
            name: "echo".into(),
            version: None,
        },
    ]);
    assert_eq!(
        kept,
        vec![
            Versioned {
                name: "add".into(),
                version: Some("10".into()),
            },
            Versioned {
                name: "echo".into(),
                version: None,
            },
        ]
    );
}

#[test]
fn memory_and_file_stores_round_trip() {
    let mut memory = MemoryStore::new();
    memory.set("banner", "ok");
    assert_eq!(memory.get("banner").as_deref(), Some("ok"));

    let path = std::env::temp_dir().join(format!("fastmcp-store-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut file = FileStore::open(&path).expect("open");
    file.set("banner", "ok");
    let again = FileStore::open(&path).expect("reopen");
    assert_eq!(again.get("banner").as_deref(), Some("ok"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_span_returns_the_body_and_the_fingerprint_is_sha256() {
    let body = traced_call("add", || "ok".to_string());
    assert_eq!(body, "ok");
    assert_eq!(FINGERPRINT_ALGO, "sha256");
    let digest = fingerprint("add", r#"{"type":"object"}"#);
    assert_eq!(digest.len(), 64);
    assert!(digest.chars().all(|ch| ch.is_ascii_hexdigit()));
}
