use fastmcp_core::{jwks_kid, sign, verify, ResourceMetadata, AUTH_ALGORITHM};
use serde_json::json;

const SECRET: &[u8] = b"test-secret-not-a-production-key";
const AUDIENCE: &str = "http://example.test/mcp";

#[test]
fn a_signed_token_verifies_and_a_bad_one_does_not() {
    assert_eq!(AUTH_ALGORITHM, "HS256");
    let token = sign(SECRET, AUDIENCE, "ada", 4_102_444_800).expect("sign");
    assert_eq!(verify(SECRET, &token, AUDIENCE).as_deref(), Ok("ada"));

    let wrong_audience = verify(SECRET, &token, "http://other.test/mcp");
    assert!(wrong_audience.is_err());
    assert_eq!(wrong_audience.unwrap_err(), "token rejected");

    let mut tampered = token.clone();
    tampered.push('x');
    let bad = verify(SECRET, &tampered, AUDIENCE).unwrap_err();
    assert_eq!(bad, "token rejected");
    assert!(!bad.contains("secret"));
}

#[test]
fn a_jwks_document_yields_the_first_kid() {
    let document = json!({"keys": [{"kid": "test-key", "kty": "oct"}]});
    assert_eq!(jwks_kid(&document).as_deref(), Some("test-key"));
    assert_eq!(jwks_kid(&json!({})), None);
}

#[test]
fn resource_metadata_names_the_authorization_server() {
    let metadata = ResourceMetadata::new(
        "http://example.test/mcp",
        vec!["http://auth.example.test".to_string()],
        vec!["tools".to_string()],
    );
    assert_eq!(metadata.resource, "http://example.test/mcp");
    assert_eq!(
        metadata.authorization_servers,
        vec!["http://auth.example.test"]
    );
    assert_eq!(metadata.scopes_supported, vec!["tools"]);
}
