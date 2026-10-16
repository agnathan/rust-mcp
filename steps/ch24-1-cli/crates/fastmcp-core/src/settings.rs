//! Settings from defaults, a JSON file, and `FASTMCP_` environment variables.
//!
//! `inspect` lists tool names and does not bind a port. `dev` sets reload.
//! `install` writes a JSON snippet.

use std::path::Path;
use std::sync::OnceLock;

use clap::{Parser, Subcommand};
use figment::{
    providers::{Env, Format, Json, Serialized},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::FastMcp;

/// Environment variables for settings start with this prefix.
pub const SETTINGS_ENV_PREFIX: &str = "FASTMCP_";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub host: String,
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            log_level: "INFO".into(),
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// The process-wide settings. The first call wins.
pub fn cached_settings() -> &'static Settings {
    SETTINGS.get_or_init(|| load_settings(None).expect("settings"))
}

/// Defaults, then an optional JSON file, then `FASTMCP_*` environment variables.
pub fn load_settings(config_path: Option<&Path>) -> Result<Settings, String> {
    let mut figment = Figment::new().merge(Serialized::defaults(Settings::default()));
    if let Some(path) = config_path {
        figment = figment.merge(Json::file(path));
    }
    figment = figment.merge(Env::prefixed(SETTINGS_ENV_PREFIX));
    figment.extract().map_err(|error| error.to_string())
}

#[derive(Parser, Debug)]
#[command(name = "fastmcp")]
struct Cli {
    #[command(subcommand)]
    command: AppCommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
enum AppCommand {
    /// Run with the configured host. This parse does not bind a socket.
    Run,
    /// Development mode. Reload is on.
    Dev,
    /// List tool names. Does not bind a port.
    Inspect,
    /// A JSON snippet a client can install.
    Install { name: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    pub command: &'static str,
    pub host: String,
    pub log_level: String,
    pub reload: bool,
    pub detail: String,
}

/// Parse `args` as if they followed the program name. Inspect does not open a socket.
pub fn plan(args: &[&str], config_path: Option<&Path>) -> Result<Plan, String> {
    let cli = Cli::try_parse_from(std::iter::once("fastmcp").chain(args.iter().copied()))
        .map_err(|error| error.to_string())?;
    let settings = load_settings(config_path)?;
    let (command, reload, detail) = match cli.command {
        AppCommand::Run => ("run", false, settings.host.clone()),
        AppCommand::Dev => ("dev", true, settings.host.clone()),
        AppCommand::Inspect => {
            let names = FastMcp::new("widgets").tool_names().join(", ");
            ("inspect", false, names)
        }
        AppCommand::Install { name } => ("install", false, install_json(&name)),
    };
    Ok(Plan {
        command,
        host: settings.host,
        log_level: settings.log_level,
        reload,
        detail,
    })
}

/// The snippet `install` would write.
pub fn install_json(name: &str) -> String {
    serde_json::json!({
        "mcpServers": {
            name: {
                "command": "fastmcp",
                "args": ["run"]
            }
        }
    })
    .to_string()
}
