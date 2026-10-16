//! Dep holds a value and a PhantomData marker. No crates beyond std.

use std::marker::PhantomData;

struct Dep<T> {
    value: T,
    _marker: PhantomData<fn() -> T>,
}

fn main() {
    let db = Dep {
        value: "main",
        _marker: PhantomData,
    };
    let token = Dep {
        value: "secret",
        _marker: PhantomData,
    };
    println!("who: {} {} widgets", db.value, token.value);
    println!("required: q");
}
