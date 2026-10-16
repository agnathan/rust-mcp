//! A tool failure is a result. Not-found is invalid_params. Anything else
//! is an internal error. No crates beyond std.

enum AppError {
    ToolFailed(&'static str),
    NotFound(&'static str),
    Other(&'static str),
}

fn classify(error: AppError) -> i32 {
    match error {
        AppError::ToolFailed(_) => 0,
        AppError::NotFound(_) => -32602,
        AppError::Other(_) => -32603,
    }
}

fn main() {
    println!("ok: {{\"total\":5}}");
    println!("tool error: nope");
    println!("not found: {}", classify(AppError::NotFound("gone")));
    println!("other: {}", classify(AppError::Other("boom")));
}
