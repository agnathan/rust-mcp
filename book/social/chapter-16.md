# Social posts: Chapter 16, "Middleware"

## X

Chapter 16 of Learning Rust by Building FastMCP is up: five hooks around one echo.

A log, a timer, a token bucket, a cache, and a size limit. The cache skips the tool the second time. A ping interval of zero is an error, and one connection gets one task.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 16 of Learning Rust by Building FastMCP is out: middleware.

FastMCP chains hooks for logging, timing, rate limiting, response caching, and response limiting, and PingMiddleware keeps a connection alive on a positive interval. The Rust chapter is a `ToolHook` list. `before` runs outer-first. `after` runs in reverse. A stopped call never reaches the tool. The ping schedule records one task per connection id and does not start the timer loop yet.

[personal note]

Written for a reader who is new to Rust. A trait object is how a `Vec` holds five different hooks.

#Rust #MCP #LearningInPublic #SoftwareEngineering
