fn allowed(src: &str) -> bool {
    src.starts_with("https://") || src.starts_with("http://") || src.starts_with("data:")
}

fn main() {
    let src = "https://example.test/icon.png";
    assert!(allowed(src));
    assert!(!allowed("javascript:alert(1)"));
    println!("name: widgets");
    println!("website: https://example.test");
    println!("src: {src}");
    println!("theme: light");
}
