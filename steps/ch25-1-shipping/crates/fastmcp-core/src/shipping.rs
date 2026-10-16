//! The metadata a reader would publish, and one in-process tool call.
//!
//! `cargo publish` is not run. The step crate stays `publish = false`.

use schemars::JsonSchema;
use semver::Version;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::server::FastMcp;
use crate::tools::ToolError;

/// The license string on FastMCP's `pyproject.toml`.
pub const CRATE_LICENSE: &str = "Apache-2.0";

/// The command this chapter plans. It is not executed.
pub const PUBLISH_COMMAND: &str = "cargo publish --dry-run";

/// `package.metadata.docs.rs` rustc cfg.
pub const fn docs_rs_cfg() -> &'static str {
    "--cfg docsrs"
}

/// Name, version, and the fields a crates.io upload needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub license: String,
    pub description: String,
    pub repository: String,
    pub rust_version: String,
}

/// A plan. `command` is the dry run. Nothing is uploaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPlan {
    pub command: String,
    pub docs_rs: String,
}

impl Package {
    /// The record this book would copy into a crate that is allowed to publish.
    pub fn book() -> Self {
        Self {
            name: "fastmcp-core".to_string(),
            version: "0.1.0".to_string(),
            license: CRATE_LICENSE.to_string(),
            description: "An MCP server in the shape of FastMCP.".to_string(),
            repository: "https://github.com/agnathan/rust-mcp".to_string(),
            rust_version: "1.88".to_string(),
        }
    }
}

/// Parse `MAJOR.MINOR.PATCH`. Pre-release and build metadata are refused.
///
/// # Errors
///
/// The `semver` crate rejects a version that is not `MAJOR.MINOR.PATCH`,
/// including a leading `v` and an empty string.
pub fn parse_release(version: &str) -> Result<Version, String> {
    let parsed = Version::parse(version).map_err(|err| err.to_string())?;
    if !parsed.pre.is_empty() || !parsed.build.is_empty() {
        return Err("release version must be MAJOR.MINOR.PATCH".to_string());
    }
    Ok(parsed)
}

/// Check the package and return the dry-run command.
///
/// # Errors
///
/// `0.0.0` is refused. An empty license, an empty description, a name with
/// an uppercase letter, and a version `parse_release` rejects are refused.
pub fn publish_plan(package: &Package) -> Result<PublishPlan, String> {
    if package.name.is_empty() || package.name.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err("crate name must be lowercase".to_string());
    }
    if package.license.is_empty() {
        return Err("license is required".to_string());
    }
    if package.description.is_empty() {
        return Err("description is required".to_string());
    }
    let version = parse_release(&package.version)?;
    if version.major == 0 && version.minor == 0 && version.patch == 0 {
        return Err("refusing to publish 0.0.0".to_string());
    }
    Ok(PublishPlan {
        command: PUBLISH_COMMAND.to_string(),
        docs_rs: docs_rs_cfg().to_string(),
    })
}

/// Archive names in the shape cargo-dist uses. Nothing is built.
pub fn release_archives(name: &str, version: &str) -> Vec<String> {
    vec![
        format!("{name}-{version}-x86_64-unknown-linux-gnu.tar.gz"),
        format!("{name}-{version}-aarch64-apple-darwin.tar.gz"),
        format!("{name}-{version}-x86_64-pc-windows-msvc.zip"),
    ]
}

#[derive(Deserialize, JsonSchema)]
struct AddArgs {
    a: i64,
    b: i64,
}

/// A server with one `add` tool. Built outside the benchmark timer.
pub fn add_server() -> FastMcp {
    FastMcp::new("widgets").tool("add", "add two integers", |args: AddArgs| {
        Ok(args.a + args.b)
    })
}

/// One in-process call. Middleware hooks are not on this path.
///
/// # Errors
///
/// [`ToolError::Unknown`] if `add` was not registered, or the argument error
/// from [`FastMcp::dispatch_tool`].
pub fn call_add(server: &FastMcp) -> Result<Value, ToolError> {
    server.dispatch_tool("add", json!({"a": 2, "b": 3}))
}
