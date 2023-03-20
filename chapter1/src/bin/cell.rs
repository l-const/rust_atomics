fn main() {
    println!("Hello, world!");
    use std::cell::Cell;


}

fn f(a: &std::cell::Cell<i32>, b: &std::cell::Cell<i32>) {
    let before = a.get();
    b.set(b.get() + 1);
    let after = a.get();
    if before != after {
        x(); // might happen
    }
}
