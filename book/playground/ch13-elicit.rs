//! Walk the elicitation phases. No runtime, no crates beyond std.

fn main() {
    let mut phase = "idle";
    println!("ask: Ship it?");
    phase = "waiting";
    let answer = "yes";
    phase = "accepted";
    println!("answer: {answer}");
    println!("phase: {phase}");
}
