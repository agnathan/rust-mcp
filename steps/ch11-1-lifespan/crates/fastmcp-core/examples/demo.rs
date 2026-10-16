//! Open a database, then a cache. Drop closes the cache first.
//! Does not open stdio.

use fastmcp_core::Lifespan;

fn main() {
    let life = Lifespan::default();
    let log = life.log_handle();
    life.enter("db");
    life.enter("cache");
    life.state().set("ready");
    println!("state: {}", life.state().get());
    drop(life);
    for line in log.lock().expect("log").iter() {
        println!("{line}");
    }
}
