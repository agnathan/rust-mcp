# Social posts: Appendix A, "Apps"

## X

Appendix A of Learning Rust by Building FastMCP is up: a tool the model sees, and a tool it does not.

`show` opens the view. `save` is `deb87552a0f1_save`. The hash is SHA-256 of the app name, a NUL, and the tool name.

[personal note]

#rustlang #MCP

## LinkedIn

Appendix A of Learning Rust by Building FastMCP is out: interactive apps.

FastMCP's `FastMCPApp` marks backend tools with visibility `app` and entry points with visibility `model`. The Rust appendix stores that split, hashes the backend name the way `hash_tool` does, and defaults a `ui://` resource to `text/html;profile=mcp-app`. Prefab, the iframe policy, and the HTML renderer stay deferred.

[personal note]

Written for a reader who is new to Rust. The NUL byte in the hash is what keeps the app name and the tool name from running together.

#Rust #MCP #LearningInPublic #SoftwareEngineering
