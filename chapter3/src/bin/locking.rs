use std::sync::atomic::AtomicBool;

use std::thread;

static mut DATA: String = String::new();
static LOCKED: AtomicBool = AtomicBool::new(false);

fn f() {
    let mut changed = false;
    thread::sleep(std::time::Duration::from_millis(3000));
    while !changed  && LOCKED.compare_exchange(false, true, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed).is_ok() {
        // Safety: We hold the exclusive lock, so nothing else is accessing DATA
        unsafe { DATA.push('!')};
        changed = true;
        LOCKED.store(false, std::sync::atomic::Ordering::Release);
    }
}

fn main() {
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(f);
        }
    });

    println!("{}", unsafe { &DATA.len()  });
}