# Social posts: Chapter 9, "Context"

## X

Chapter 9 of Learning Rust by Building FastMCP is up: a Context inside a tool.

The tool logs, reports progress, and reads a resource. None of that is a parameter. The schema is still just the argument struct. `Context` is `Clone` because it holds an `Arc`, and the notes sit behind a `Mutex` so a test can read them after the call.

Sampling returns a fixed string here. The real request to a model waits.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 9 of Learning Rust by Building FastMCP is out: `Context`.

FastMCP injects a `Context` into a tool that asks for one. The tool can log, report progress, and read a resource without those values appearing in the JSON schema. The Rust chapter does the same with `Arc` and `Mutex`. Cloning the context shares the notes. The bench records a thousand progress values in memory, with no socket in the timer.

[personal note]

Written for a reader who is new to Rust. `Arc` and `Send + Sync` show up here because the tool and the test share one list.

#Rust #MCP #LearningInPublic #SoftwareEngineering
