use fastmcp_core::{namespace_name, Catalog};

fn main() {
    let mut catalog = Catalog::new(vec![("add", vec!["public"]), ("secret", vec!["internal"])]);
    let name = namespace_name("math", "add");
    catalog.disable_tag("internal");
    let visible = catalog.visible().join(", ");
    catalog.prompts_as_tools();
    println!("name: {name}");
    println!("visible: {visible}");
    println!("prompts: list_prompts, get_prompt");
    println!("changed: {}", catalog.list_changed());
}
