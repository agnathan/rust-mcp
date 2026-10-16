# Social posts: Chapter 7, "Prompts, completions, and pagination"

## X

Chapter 7 of Learning Rust by Building FastMCP is up: prompts, completions, and a cursor.

`greet` with the name Ada becomes `Hello, Ada`. While you type `A`, the server offers Ada and Alan, and it stops at 100 suggestions. `list_*` no longer sends the whole catalog. The cursor is base64 of `{"o": 2}`. The client sends it back. It does not parse it.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 7 of Learning Rust by Building FastMCP is out: prompts, argument completion, and cursor pagination.

FastMCP encodes a list cursor as URL-safe base64 of an offset, and it turns a bad cursor into `INVALID_PARAMS`. A completion is capped at 100 values. The Rust chapter does both. A prompt is a function from an argument map to a message. `page_items` returns `impl Iterator`, so the caller never has to name the `Skip` and `Take` types underneath the page.

The bench splits the two costs. Rendering one prompt is tens of nanoseconds. Taking one page from the middle of 10,000 items is a couple of microseconds, and that timer does not include a network round trip.

[personal note]

Written for a reader who is new to Rust. Iterators show up here because a page is a slice of a longer list.

#Rust #MCP #LearningInPublic #SoftwareEngineering
