# Social posts: Chapter 13, "Elicitation and interactive tools"

## X

Chapter 13 of Learning Rust by Building FastMCP is up: a tool that waits for a person.

`ask` stores a oneshot sender and sets the phase to waiting. Another task submits "yes". The phase becomes accepted. A nested object in the form schema is rejected before anyone is asked.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 13 of Learning Rust by Building FastMCP is out: elicitation and interactive tools.

FastMCP validates a flat elicitation schema and handles an accepted answer. The Rust chapter uses an enum for the phase and a oneshot channel for the single answer. Decline comes back as a tool result with `is_error`. The SEP-2322 multi-round wire protocol waits.

[personal note]

Written for a reader who is new to Rust. A oneshot is one message, and then the sender is finished.

#Rust #MCP #LearningInPublic #SoftwareEngineering
