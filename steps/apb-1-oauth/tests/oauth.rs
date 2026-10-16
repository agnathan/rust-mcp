use mcp_oauth::{
    code_challenge_s256, oidc_from_json, MemoryOAuth, OAuthProxy, AUTH_CODE_EXPIRY_SECONDS,
    PKCE_METHOD,
};

const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

#[test]
fn a_code_becomes_a_token_when_the_verifier_matches() {
    let challenge = code_challenge_s256(VERIFIER);
    let mut server = MemoryOAuth::new("http://auth.example.test");
    server.register("mcp", "http://client.example.test/callback");
    let code = server
        .authorize(
            "mcp",
            "http://client.example.test/callback",
            &challenge,
            1_000,
        )
        .expect("code");
    let token = server.exchange(&code, VERIFIER, 1_000).expect("token");
    assert_eq!(token, "tok-1");
    assert!(server.exchange(&code, VERIFIER, 1_000).is_err());
    assert_eq!(AUTH_CODE_EXPIRY_SECONDS, 300);
    assert_eq!(PKCE_METHOD, "S256");
    let meta = server.metadata();
    assert_eq!(meta["issuer"], "http://auth.example.test");
    assert_eq!(meta["code_challenge_methods_supported"][0], "S256");
}

#[test]
fn an_expired_code_is_refused() {
    let challenge = code_challenge_s256(VERIFIER);
    let mut server = MemoryOAuth::new("http://auth.example.test");
    server.register("mcp", "http://client.example.test/callback");
    let code = server
        .authorize(
            "mcp",
            "http://client.example.test/callback",
            &challenge,
            1_000,
        )
        .expect("code");
    let err = server
        .exchange(&code, VERIFIER, 1_000 + AUTH_CODE_EXPIRY_SECONDS)
        .expect_err("expired");
    assert_eq!(err, "code expired");
}

#[test]
fn the_proxy_keeps_the_client_redirect_and_returns_the_upstream_token() {
    let challenge = code_challenge_s256(VERIFIER);
    let mut proxy = OAuthProxy::new(
        "https://idp.example.test/authorize",
        "https://proxy.example.test/callback",
    );
    let url = proxy.start("http://127.0.0.1:9/cb", "client-state", &challenge);
    assert!(url.contains("state=tx-1"));
    assert!(url.contains("redirect_uri=https://proxy.example.test/callback"));
    let back = proxy.complete("tx-1", "abc").expect("complete");
    assert!(back.starts_with("http://127.0.0.1:9/cb?code="));
    assert!(back.contains("state=client-state"));
    let code = back
        .split("code=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();
    assert_eq!(
        proxy.exchange(code, VERIFIER).expect("token"),
        "upstream:abc"
    );
}

#[test]
fn strict_oidc_requires_the_discovery_fields() {
    let text = r#"{
        "issuer": "http://idp.example.test",
        "authorization_endpoint": "http://idp.example.test/authorize",
        "token_endpoint": "http://idp.example.test/token",
        "jwks_uri": "http://idp.example.test/jwks",
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"]
    }"#;
    let config = oidc_from_json(text, true).expect("oidc");
    assert_eq!(config.issuer, "http://idp.example.test");
    let err = oidc_from_json(r#"{"issuer":"http://idp.example.test"}"#, true).expect_err("strict");
    assert!(err.contains("Missing required configuration metadata:"));
}
