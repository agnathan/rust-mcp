//! A positive ping interval, and a reply cut down to five bytes.

fn main() {
    let interval_ms = 30_000;
    println!("layers: 5");
    println!("body: hello");
    if interval_ms > 0 {
        println!("ping: {interval_ms}");
    }
    let mut body = String::from("hello world");
    body.truncate(5);
    println!("cut: {body}");
}
