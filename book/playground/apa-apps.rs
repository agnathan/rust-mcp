fn main() {
    let tools = [("show", "model"), ("save", "app")];
    let model: Vec<_> = tools
        .iter()
        .filter(|(_, audience)| *audience == "model")
        .map(|(name, _)| *name)
        .collect();
    println!("extension: io.modelcontextprotocol/ui");
    println!("model: {}", model.join(", "));
    println!("hashed: deb87552a0f1_save");
    println!("body: saved");
}
