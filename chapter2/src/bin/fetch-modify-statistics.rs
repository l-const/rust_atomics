use std::{sync::atomic::{AtomicUsize, AtomicU64}, time::Instant, num};

fn main() {
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);

    std::thread::scope(|s| {
        // Four background threads to process al 100 items, 25 each thread 
        for t in 0..4 {
            s.spawn(move || {
                for i in 0..25 {
                    let start = Instant::now();
                    process_item(t * 25 + i);
                    let time_taken = start.elapsed().as_micros() as u64;
                    num_done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    total_time.fetch_add(time_taken, std::sync::atomic::Ordering::Relaxed);
                    max_time.fetch_max(time_taken, std::sync::atomic::Ordering::Relaxed);
                }
            });
        }
        // The main thread shows status updates, every second.
        loop {
            let n = num_done.load(std::sync::atomic::Ordering::Relaxed);
            let total_time = std::time::Duration::from_micros(total_time.load(std::sync::atomic::Ordering::Relaxed));
            let max_time = std::time::Duration::from_micros(max_time.load(std::sync::atomic::Ordering::Relaxed));
            println!("Num done: {n}, total time: {total_time:?}, max time: {max_time:?}");
            if n == 0 {
                println!("Working ... nothing to be done.");
            } else {
                println!("Working.. {n}/100 done , {:?} average, {:?} peak", total_time / n  as u32, max_time)
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    })
}



fn process_item(x: i32) {
    std::thread::sleep(std::time::Duration::from_millis(100));
    dbg!("Processing item: {}", x);
}