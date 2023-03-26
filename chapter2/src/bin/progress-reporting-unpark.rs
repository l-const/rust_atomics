use std::sync::atomic::{Ordering::Relaxed, AtomicUsize};
use std::thread;
fn main() {
    let num_done = AtomicUsize::new(0);

    let main_thread_handle = thread::current();
    thread::scope(|s| {
        // A background thread to process 100 items
        s.spawn(|| {
            for i in 0..100 {
                process_item(i); // Assuming this takes some time
                num_done.store(i + 1, Relaxed);
                main_thread_handle.unpark();
            }
        });

        // the main thread show status updates, every second.
        loop {
            let n  = num_done.load(Relaxed);
            if n == 100 { break;}
            println!("Working..{n}/100 done");
            thread::park_timeout(std::time::Duration::from_secs(1));
        }
    });

    println!("Done!");
    
}

fn process_item(i: usize) {
    thread::sleep(std::time::Duration::from_millis(200));
}