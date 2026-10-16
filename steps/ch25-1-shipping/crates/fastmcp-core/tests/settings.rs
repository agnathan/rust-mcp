use std::fs;
use std::sync::Mutex;

use fastmcp_core::{install_json, load_settings, plan, SETTINGS_ENV_PREFIX};

fn without_overrides<T>(body: impl FnOnce() -> T) -> T {
    static LOCK: Mutex<()> = Mutex::new(());
    let _guard = LOCK.lock().expect("settings lock");
    let host = std::env::var("FASTMCP_HOST").ok();
    let level = std::env::var("FASTMCP_LOG_LEVEL").ok();
    unsafe {
        std::env::remove_var("FASTMCP_HOST");
        std::env::remove_var("FASTMCP_LOG_LEVEL");
    }
    let value = body();
    unsafe {
        match host {
            Some(host) => std::env::set_var("FASTMCP_HOST", host),
            None => std::env::remove_var("FASTMCP_HOST"),
        }
        match level {
            Some(level) => std::env::set_var("FASTMCP_LOG_LEVEL", level),
            None => std::env::remove_var("FASTMCP_LOG_LEVEL"),
        }
    }
    value
}

#[test]
fn defaults_then_a_file_then_the_environment() {
    without_overrides(|| {
        let settings = load_settings(None).expect("defaults");
        assert_eq!(settings.host, "127.0.0.1");
        assert_eq!(settings.log_level, "INFO");
        assert_eq!(SETTINGS_ENV_PREFIX, "FASTMCP_");

        let path =
            std::env::temp_dir().join(format!("fastmcp-settings-{}.json", std::process::id()));
        fs::write(&path, r#"{"host":"10.0.0.1","log_level":"DEBUG"}"#).expect("write");
        let from_file = load_settings(Some(&path)).expect("file");
        assert_eq!(from_file.host, "10.0.0.1");
        assert_eq!(from_file.log_level, "DEBUG");

        unsafe {
            std::env::set_var("FASTMCP_HOST", "10.1.1.1");
        }
        let from_env = load_settings(Some(&path)).expect("env");
        assert_eq!(from_env.host, "10.1.1.1");
        assert_eq!(from_env.log_level, "DEBUG");
        unsafe {
            std::env::remove_var("FASTMCP_HOST");
        }
        let _ = fs::remove_file(&path);
    });
}

#[test]
fn inspect_lists_tools_and_dev_turns_reload_on() {
    without_overrides(|| {
        let inspected = plan(&["inspect"], None).expect("inspect");
        assert_eq!(inspected.command, "inspect");
        assert_eq!(inspected.detail, "status");
        assert!(!inspected.reload);
        assert_eq!(inspected.host, "127.0.0.1");

        let dev = plan(&["dev"], None).expect("dev");
        assert!(dev.reload);
        assert_eq!(dev.command, "dev");

        let snippet = install_json("widgets");
        assert!(snippet.contains("widgets"));
        assert!(snippet.contains("fastmcp"));
        let installed = plan(&["install", "widgets"], None).expect("install");
        assert_eq!(installed.detail, snippet);
    });
}
