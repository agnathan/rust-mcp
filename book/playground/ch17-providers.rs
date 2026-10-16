fn mount(namespace: &str, name: &str) -> String {
    format!("{namespace}_{name}")
}

fn instructions<'a>(own: Option<&'a str>, upstream: Option<&'a str>) -> Option<&'a str> {
    own.or(upstream)
}

fn main() {
    let api = mount("api", "status");
    println!("tools: {api}, local_status");
    println!("ping: ok");
    let text = instructions(None, Some("be brief")).unwrap_or("");
    println!("instructions: {text}");
}
