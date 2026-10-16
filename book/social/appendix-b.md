# Social posts: Appendix B, "OAuth"

## X

Appendix B of Learning Rust by Building FastMCP is up: a code, a proxy, and a discovery document.

S256, 300 seconds, one use. The proxy keeps the client's redirect and stores `upstream:abc`. No socket.

[personal note]

#rustlang #MCP

## LinkedIn

Appendix B of Learning Rust by Building FastMCP is out: the OAuth server, the OAuth proxy, and OIDC discovery.

Chapter 22 verified a token. This appendix issues one in memory, with S256 PKCE and a five-minute code. The proxy uses a fixed upstream redirect and returns the client's own redirect with a new code. Strict OIDC mode refuses a document that is missing one of the seven fields FastMCP checks. Consent pages, real HTTP, and the vendor providers stay deferred.

[personal note]

Written for a reader who is new to Rust. The code comes out of the map before the verifier is checked, so a retry cannot spend it twice.

#Rust #MCP #LearningInPublic #SoftwareEngineering
