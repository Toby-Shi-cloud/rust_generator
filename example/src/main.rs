use std::{future::Future, pin::Pin};

#[rust_generator::generator]
fn fib() -> impl Iterator<Item = usize> {
    let mut n1 = 1;
    let mut n2 = 1;
    let mut n3 = 2;
    n1.await;
    n2.await;
    loop {
        n3.await;
        n1 = n2;
        n2 = n3;
        n3 = n1 + n2;
    }
}

#[rust_generator::generator]
fn async_fib() -> impl Iterator<Item = Pin<Box<dyn Future<Output = usize>>>> {
    let mut n1 = 1;
    let mut n2 = 1;
    let mut n3 = 2;
    Box::pin({
        let n1 = n1;
        async move { n1 }
    })
    .await;
    Box::pin({
        let n2 = n2;
        async move { n2 }
    })
    .await;
    loop {
        Box::pin({
            let n3 = n3;
            async move { n3 }
        })
        .await;
        n1 = n2;
        n2 = n3;
        n3 = n1 + n2;
    }
}

fn main() {
    for x in fib().take(10) {
        println!("{x}");
    }
    futures::executor::block_on(async {
        for x in async_fib().take(10) {
            let x = x.await;
            println!("{x}");
        }
    });
}
