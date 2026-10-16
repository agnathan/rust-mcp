fn main() {
    let handle = std::thread::spawn(|| "ok");
    let id = 1u64;
    let result = handle.join().expect("task");
    println!("id: {id}");
    println!("status: done");
    println!("result: {result}");
}
