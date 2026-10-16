//! One page of names, and the cursor that points at the next page.
//! The cursor is URL-safe base64 of {"o": offset}. No crates beyond std.

fn encode_cursor(offset: usize) -> String {
    b64(format!(r#"{{"o":{offset}}}"#).as_bytes())
}

fn b64(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut index = 0;
    while index + 3 <= data.len() {
        let chunk = ((data[index] as u32) << 16)
            | ((data[index + 1] as u32) << 8)
            | (data[index + 2] as u32);
        out.push(ALPHABET[((chunk >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 63) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 63) as usize] as char);
        out.push(ALPHABET[(chunk & 63) as usize] as char);
        index += 3;
    }
    let rest = data.len() - index;
    if rest == 1 {
        let chunk = (data[index] as u32) << 16;
        out.push(ALPHABET[((chunk >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rest == 2 {
        let chunk = ((data[index] as u32) << 16) | ((data[index + 1] as u32) << 8);
        out.push(ALPHABET[((chunk >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 63) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

fn main() {
    let name = "Ada";
    println!("greet: Hello, {name}");
    let names = ["greet", "one", "three", "two"];
    let page: Vec<&str> = names.iter().copied().take(2).collect();
    println!("page: {}", page.join(", "));
    println!("next: {}", encode_cursor(2));
}
