use std::{sync::atomic::{Ordering::{Release, Acquire}, AtomicU64, AtomicBool}, thread};
use std::sync::atomic::Ordering::Relaxed;
static DATA: AtomicU64 = AtomicU64::new(0);
static READY: AtomicBool = AtomicBool::new(false);

fn main() {
    thread::spawn(|| {
        DATA.store(123, Relaxed);
        READY.store(true, Release); // everything from before this store
    });

    while !READY.load(Acquire) {
        thread::sleep(std::time::Duration::from_millis(100));
        println!("waiting...")
    }

    println!("{}", DATA.load(Relaxed));
}