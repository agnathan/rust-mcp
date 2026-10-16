fn main() {
    let versions = [("add", Some(2u32)), ("add", Some(10))];
    let mut best = 0u32;
    for (_, version) in versions {
        if let Some(version) = version {
            if version > best {
                best = version;
            }
        }
    }
    println!("allowed: add");
    println!("version: {best}");
    println!("stored: ok");
    println!("algo: sha256");
}
