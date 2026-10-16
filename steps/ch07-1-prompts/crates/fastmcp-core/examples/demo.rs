//! Render one prompt, and show the first page of a four-name catalog.
//! Does not open stdio.

use std::collections::HashMap;

use fastmcp_core::{paginate_sequence, FastMcp};

fn main() {
    let server =
        FastMcp::new("fastmcp-rs").prompt("greet", "Greet someone", &[("name", true)], |args| {
            let name = args.get("name").map(String::as_str).unwrap_or("there");
            Ok(format!("Hello, {name}"))
        });
    let mut arguments = HashMap::new();
    arguments.insert("name".into(), "Ada".into());
    let text = server.render_prompt("greet", &arguments).expect("greet");
    println!("greet: {text}");

    let names = ["greet", "one", "three", "two"];
    let page = paginate_sequence(&names, None, 2).expect("page");
    println!("page: {}", page.items.join(", "));
    println!("next: {}", page.next_cursor.expect("cursor"));
}
