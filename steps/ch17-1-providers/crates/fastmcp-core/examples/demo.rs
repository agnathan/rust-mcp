use std::sync::Arc;

use fastmcp_core::{AggregateProvider, LocalProvider, MountedProvider, Provider, ProxyProvider};

fn main() {
    let inner: Arc<dyn Provider> =
        Arc::new(LocalProvider::new(vec![("status", "ok")]).with_instructions("be brief"));
    let local: Arc<dyn Provider> = Arc::new(MountedProvider::new("local", inner.clone()));
    let api: Arc<dyn Provider> = Arc::new(MountedProvider::new(
        "api",
        Arc::new(LocalProvider::new(vec![("status", "api-ok")])),
    ));
    let combined = AggregateProvider::new(vec![api, local]);
    let proxy = ProxyProvider::new(inner);

    let names = combined.list_tools().join(", ");
    let ping = match proxy.ping() {
        Ok(()) => "ok",
        Err(_) => "err",
    };
    let instructions = proxy.instructions().unwrap_or_default();
    println!("tools: {names}");
    println!("ping: {ping}");
    println!("instructions: {instructions}");
}
