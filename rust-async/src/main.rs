#![allow(dead_code, unused_variables, unused_imports)]

use std::future::Future;


fn main() {
    println!("Hello, world!");

    let x = foo2().await;
}

async fn foo1() -> usize {
    println!("foo1");
    0
}

fn foo2() -> impl Future<Output = usize> {
    async {
        println!("foo2");
        0
    }
}