# Social posts: Chapter 25, "Shipping it"

## X

Chapter 25 of Learning Rust by Building FastMCP is up: a publish plan, and one tool call next to Python.

`0.0.0` is refused. The command is `cargo publish --dry-run`, and it is not run. `add(2, 3)` is 5 on both sides.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 25 of Learning Rust by Building FastMCP is out: shipping.

FastMCP's package version comes from git, and the fallback string is `0.0.0`. The Rust chapter parses `MAJOR.MINOR.PATCH`, refuses that fallback, and returns `cargo publish --dry-run` without spawning cargo. Three archive names follow the cargo-dist shape. Nothing is uploaded. The same `add(2, 3)` call is timed against FastMCP with middleware off.

[personal note]

Written for a reader who is new to Rust. `semver::Version` is how the three numbers stay three numbers.

#Rust #MCP #LearningInPublic #SoftwareEngineering
