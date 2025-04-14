#[gen_macro::generator]
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
