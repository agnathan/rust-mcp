use std::borrow::Cow;

fn namespace_name<'a>(prefix: &str, name: &'a str) -> Cow<'a, str> {
    if prefix.is_empty() {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(format!("{prefix}_{name}"))
    }
}

fn main() {
    let name = namespace_name("math", "add");
    println!("name: {name}");
    let tools = [("add", true), ("secret", false)];
    let visible: Vec<_> = tools
        .iter()
        .filter(|(_, enabled)| *enabled)
        .map(|(tool, _)| *tool)
        .collect();
    println!("visible: {}", visible.join(", "));
    println!("prompts: list_prompts, get_prompt");
    println!("changed: true");
}
