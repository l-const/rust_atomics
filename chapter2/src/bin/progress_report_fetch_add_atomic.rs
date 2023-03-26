use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

fn main() {
    let num_done = &AtomicUsize::new(0);
    std::thread::scope(|s| {
        // Four background threads to process all 100 items, 25 each
        for t in 0..4 {
            s.spawn(move || {
                for i in 0..25 {
                    process(t * 25 + i); // assuming this take an amount time
                    num_done.fetch_add(1, Relaxed);
                }
            });
        }

        loop {
            let n = num_done.load(Relaxed);
            if n == 100 { break;}
            println!("Working.. {n}/100 done");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

    });

    println!("Done!");
}


fn process(x: i32) {
    std::thread::sleep(std::time::Duration::from_millis(100));
    dbg!(x);
}