# Social posts: Chapter 22, "Authentication"

## X

Chapter 22 of Learning Rust by Building FastMCP is up: check the token before the tool runs.

HS256, audience must match, and the error is `token rejected`. The secret stays out of the message. A metadata struct names the authorization server. The full OAuth server waits.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 22 of Learning Rust by Building FastMCP is out: authentication.

FastMCP's JWTVerifier accepts asymmetric algorithms and HMAC, and it can refresh a JWKS document. RemoteAuthProvider advertises RFC 9728 protected-resource metadata. The Rust chapter verifies HS256, rejects a bad signature and a wrong audience without logging the secret, reads a `kid` from a JWKS document it already has, and stores the resource metadata fields. Fetching keys and the full OAuth server stay deferred.

[personal note]

Written for a reader who is new to Rust. `map_err` is where the secret stays off the error string.

#Rust #MCP #LearningInPublic #SoftwareEngineering
