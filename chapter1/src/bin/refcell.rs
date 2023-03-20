use std::cell::RefCell;

fn main() {
    println!("Hello, world!");
    use std::cell::RefCell;
    let a = RefCell::new(vec![3,4,5]);
    f(&a);
    f(&a);
    dbg!(&a);
}

fn f(v: &RefCell<Vec<i32>>) {
    v.borrow_mut().push(99);
}

