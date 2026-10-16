# Social posts: Chapter 21, "OpenAPI to MCP"

## X

Chapter 21 of Learning Rust by Building FastMCP is up: an OpenAPI document becomes tools.

`operationId` is the tool name. A call fills `{id}` and asks a caller. The tests record the URL. A reqwest client builds, and the tests do not send.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 21 of Learning Rust by Building FastMCP is out: OpenAPI to MCP.

FastMCP's OpenAPIProvider reads a spec and builds tools that perform HTTP calls. The Rust chapter parses `paths` and `operationId`, fills path parameters, and sends the request through an `HttpCaller`. Tests use a stand-in. `ReqwestCaller` is the live client, and the test only checks that it builds.

[personal note]

Written for a reader who is new to Rust. A trait is the seam that lets the test avoid a live server.

#Rust #MCP #LearningInPublic #SoftwareEngineering
