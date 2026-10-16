# FastMCP wire guardrails for this book

Source: `PrefectHQ/fastmcp`, commit `e2fcc412001ff7414095c5dc282f94c70299a612` (2026-10-08). Paths are relative to `fastmcp_slim/fastmcp/`. Only rules that were read in those files are listed. This is not a full security catalog.

## Rules seen in the source

| Id | Where | What the code does |
|---|---|---|
| W1 | `server/low_level.py` | A request is a message with an id. `initialize` is dispatched only when `ctx.method == "initialize"` and `ctx.request_id is not None`. |
| W2 | `server/low_level.py` | Notifications, including `notifications/initialized`, are not requests. They take the outer middleware path. A notification does not get a second JSON-RPC response from that branch. |
| W3 | `server/low_level.py` | If middleware raises `MCPError` after `initialize` already produced a result, FastMCP logs and returns the committed result so the client does not see two responses. |
| W4 | `exceptions.py` | `to_mcp_error` maps `NotFoundError`, `DisabledError`, and `ValidationError` to `INVALID_PARAMS`. Other exceptions use `INTERNAL_ERROR` unless the value is already an `MCPError`. |
| W5 | `server/middleware/ping.py` | `PingMiddleware` sends server-to-client pings on an interval (`interval_ms`, default `30000`). The interval must be positive. One task per connection. |
| W6 | `client/client.py` | `Client.ping` sends a ping and treats an `EmptyResult` as success. `Client.initialize` is the handshake and is idempotent. A modern `server/discover` session has no `InitializeResult`. |
| W7 | `server/providers/proxy.py` | A proxy that handles `ping` forwards it and returns `EmptyResult`. On `initialize`, it can copy the upstream `instructions` onto the result. |

Chapter 1 enforces W1, W2, and the "one of result or error" half of W3 on the hand-rolled types. Chapter 8 enforces W4. Chapter 14 enforces W6. Chapter 16 enforces W5: the interval is positive, and one connection gets one ping task. W7's proxy waits for chapter 17.
