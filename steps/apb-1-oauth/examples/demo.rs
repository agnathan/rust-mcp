use mcp_oauth::{code_challenge_s256, oidc_from_json, MemoryOAuth, OAuthProxy, PKCE_METHOD};

fn main() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = code_challenge_s256(verifier);
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
    let token = server.exchange(&code, verifier, 1_000).expect("token");

    let mut proxy = OAuthProxy::new(
        "https://idp.example.test/authorize",
        "https://proxy.example.test/callback",
    );
    proxy.start("http://127.0.0.1:9/cb", "client-state", &challenge);
    proxy.complete("tx-1", "abc").expect("complete");
    let upstream = proxy.exchange("client-code-2", verifier).expect("upstream");

    let oidc = oidc_from_json(
        r#"{"issuer":"http://idp.example.test","authorization_endpoint":"http://idp.example.test/authorize","token_endpoint":"http://idp.example.test/token","jwks_uri":"http://idp.example.test/jwks","response_types_supported":["code"],"subject_types_supported":["public"],"id_token_signing_alg_values_supported":["RS256"]}"#,
        true,
    )
    .expect("oidc");

    println!("issuer: {}", server.issuer());
    println!("method: {PKCE_METHOD}");
    println!("token: {token}");
    println!("proxy: {upstream}");
    println!("oidc: {}", oidc.issuer);
}
