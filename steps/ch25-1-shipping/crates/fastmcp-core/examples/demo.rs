use fastmcp_core::{add_server, call_add, publish_plan, release_archives, Package};

fn main() {
    let package = Package::book();
    let plan = publish_plan(&package).expect("plan");
    let targets = release_archives(&package.name, &package.version);
    let value = call_add(&add_server()).expect("add");
    println!("version: {}", package.version);
    println!("command: {}", plan.command);
    println!("targets: {}", targets.len());
    println!("rust: {value}");
}
