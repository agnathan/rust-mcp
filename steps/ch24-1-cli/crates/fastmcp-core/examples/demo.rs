use fastmcp_core::plan;

fn main() {
    let planned = plan(&["inspect"], None).expect("inspect");
    println!("host: {}", planned.host);
    println!("level: {}", planned.log_level);
    println!("command: {}", planned.command);
    println!("tools: {}", planned.detail);
}
