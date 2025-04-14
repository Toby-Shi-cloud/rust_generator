# Rust Generator

A simple generator base on async rust! No experimental needed!

## Example

Code use rust_generator:
```rust
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

fn main() {
    for x in fib().take(10) {
        println!("{x}");
    }
}
```

Code use experimental rust:
```rust
fn fib() -> impl Iterator<Item = usize> {
    gen {
        let mut n1 = 1;
        let mut n2 = 1;
        let mut n3 = 2;
        yield n1;
        yield n2;
        loop {
            yield n3;
            n1 = n2;
            n2 = n3;
            n3 = n1 + n2;
        }
    }
}
```
