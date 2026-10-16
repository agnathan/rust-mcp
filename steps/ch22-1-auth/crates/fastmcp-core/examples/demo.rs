use fastmcp_core::{sign, verify, ResourceMetadata, AUTH_ALGORITHM};

fn main() {
    let secret = b"demo-secret-not-printed";
    let audience = "http://example.test/mcp";
    let token = sign(secret, audience, "ada", 4_102_444_800).expect("sign");
    let subject = verify(secret, &token, audience).expect("verify");
    let metadata = ResourceMetadata::new(
        audience,
        vec!["http://auth.example.test".to_string()],
        vec!["tools".to_string()],
    );
    println!("alg: {AUTH_ALGORITHM}");
    println!("subject: {subject}");
    println!("resource: {}", metadata.resource);
}
