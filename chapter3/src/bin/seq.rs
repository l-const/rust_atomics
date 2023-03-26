use std::sync::atomic::{Ordering::SeqCst, AtomicBool};
//Atomic operations tagged memory_order_seq_cst not only
// order memory the same way as release/acquire ordering
// (everything that happened-before a store in one thread becomes a visible side effect
// in the thread that did a load), but also establish a single 
// total modification order of all atomic operations that are so tagged. 

static A: AtomicBool = AtomicBool::new(false);
static B: AtomicBool = AtomicBool::new(false);

static mut S: String = String::new(); 

fn main() {
     // If both store operations happen before either of the load operations,
     // it’s possible that neither thread ends up accessing S.
     // However, it’s impossible for both threads to access S and
     // cause undefined behavior, since the sequentially consistent ordering
     // guarantees only one of them can win the race. 
     //In every possible single total order,
     // the first operation will be a store operation,
     // which prevents the other thread from accessing S.
    let a = std::thread::spawn( || {
        A.store(true, SeqCst);
        if !B.load(SeqCst) {
            unsafe {S.push('!')};
        }
    });

    let b = std::thread::spawn(|| {
        B.store(true, SeqCst);
        if !A.load(SeqCst) {
            unsafe {S.push('!')};
        }
    });

    a.join().unwrap();
    b.join().unwrap();
    println!("{}", unsafe{&S} );
}