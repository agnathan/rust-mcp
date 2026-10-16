//! Print one initialize request and the response `dispatch` builds.
//! No network, no API key.

use fastmcp_core::{reply, Request, RequestId, PROTOCOL_VERSION};
use serde_json::json;

fn main() {
    let request = Request::new(
        RequestId::Number(1),
        "initialize",
        Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "demo", "version": "0.1.0"}
        })),
    );
    let bytes = serde_json::to_vec(&request).expect("serialize request");
    println!(
        "{}",
        serde_json::to_string(&request).expect("print request")
    );
    match reply(&bytes) {
        Ok(Some(response)) => {
            println!(
                "{}",
                serde_json::to_string(&response).expect("print response")
            );
        }
        Ok(None) => println!("(no response)"),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
