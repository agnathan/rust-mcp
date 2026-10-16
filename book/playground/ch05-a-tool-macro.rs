//! The shape `#[tool]` emits: a function, and a struct that forwards to it.
//! The playground does not run procedural macros. This file is the expansion.

fn add(left: i64, right: i64) -> i64 {
    left + right
}

struct AddTool;

impl AddTool {
    fn name() -> &'static str {
        "add"
    }

    fn call(left: i64, right: i64) -> i64 {
        add(left, right)
    }
}

fn main() {
    println!("{}", AddTool::call(2, 3));
    println!("{}", AddTool::name());
}
