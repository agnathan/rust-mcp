fn main() {
    let version = "0.1.0";
    assert_ne!(version, "0.0.0");
    println!("version: {version}");
    println!("command: cargo publish --dry-run");
    println!("targets: 3");
    println!("rust: {}", 2 + 3);
}
