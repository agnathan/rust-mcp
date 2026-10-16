//! A list, and a guard that puts a value back. No crates beyond std.

struct LevelGuard {
    previous: &'static str,
}

impl Drop for LevelGuard {
    fn drop(&mut self) {
        println!("level: {}", self.previous);
    }
}

fn main() {
    println!("memory: status");
    println!("duplex: status");
    let _guard = LevelGuard { previous: "INFO" };
    println!("level: DEBUG");
}
