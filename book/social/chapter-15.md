# Social posts: Chapter 15, "Testing utilities"

## X

Chapter 15 of Learning Rust by Building FastMCP is up: a test that never opens a port.

A harness lists `status` in process. The duplex from chapter 14 lists the same name through the pipe. A guard puts the log level back when it drops. The snapshot is the one word `status`.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 15 of Learning Rust by Building FastMCP is out: testing utilities.

FastMCP's in-memory transport runs the client and the server in one process, and `temporary_settings` restores a setting when the block ends. The Rust chapter splits that into a harness that calls the server directly and the duplex client from chapter 14. An inline snapshot keeps the tool list in the test. `Drop` puts the log level back, including when the test panics.

[personal note]

Written for a reader who is new to Rust. `Drop` is the place a value cleans up after itself.

#Rust #MCP #LearningInPublic #SoftwareEngineering
