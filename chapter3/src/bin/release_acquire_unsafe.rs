use std::sync::atomic::AtomicBool;

static mut DATA: u64 = 0;
static READY: AtomicBool = AtomicBool::new(false);

fn main() {
    std::thread::spawn( || {
        // Safety: Nothing else is accessing DATA,
        // because we haven't set the READY flag yet.
        unsafe{ DATA = 123};
        READY.store(true, std::sync::atomic::Ordering::Release);
        // Everything from before this store ..
    });

    while !READY.load(std::sync::atomic::Ordering::Acquire) { // .. is visible after this loads 'true'
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("waiting ...");
    }

    // Safety: Nothing is mutating DATA, because READY is set.
    println!("{}", unsafe{DATA});
}