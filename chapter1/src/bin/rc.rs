use std::{thread, borrow::Borrow};

fn main() {
    println!("Hello, world!");
    use std::rc::Rc;

    let a  = Rc::new([1,2,3]);
    let b = a.clone();

    assert_eq!(a.as_ptr(), b.as_ptr());
    use_arc()
}


fn use_arc() {
    use std::sync::Arc;

    let a = Arc::new([1, 2, 4]);
    let b = a.clone();
    let c = b.clone();
    // thread::spawn(move || dbg!(a));
    thread::spawn(move || dbg!(b));
    dbg!(Arc::strong_count(&c));
   let c =  unsafe { a.get_unchecked(4) };
   dbg!(c);

}
