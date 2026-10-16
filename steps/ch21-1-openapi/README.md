# 21.1 OpenAPI to MCP

```bash
cargo test -p ch21-1-fastmcp-core
cargo bench -p ch21-1-fastmcp-core --bench openapi
cargo run -p ch21-1-fastmcp-core --example demo
```

`tools_from_spec` reads `paths` and `operationId`. `call_operation` fills `{param}` and asks an `HttpCaller`. Tests use `MapCaller`. `ReqwestCaller` builds a client and does not send in the tests.
