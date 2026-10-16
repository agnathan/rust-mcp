//! Read one static resource and one URI template.
//! Does not open stdio.

use fastmcp_core::FastMcp;

fn main() {
    let server = FastMcp::new("fastmcp-rs")
        .resource("file://readme", "readme", "text/plain", "hello")
        .template("res://{owner}/{repo}", "repo", "text/plain", |params| {
            let owner = param(params, "owner");
            let repo = param(params, "repo");
            Ok(format!("{owner}/{repo}"))
        });
    let (readme, _) = server.read("file://readme").expect("readme");
    let (repo, _) = server.read("res://acme/widgets").expect("repo");
    println!("readme: {readme}");
    println!("repo: {repo}");
}

fn param<'a>(params: &'a [(String, String)], name: &str) -> &'a str {
    params
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
        .unwrap_or("")
}
