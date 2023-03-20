use std::{collections::VecDeque, sync::Mutex, thread, time::Instant};

// Let’s dive into an example that uses a mutex to share a queue between two threads. In the following example, a newly spawned thread will consume items from the queue, while the main thread will insert a new item into the queue every second. 
// Thread parking is used to make the 
// consuming thread wait when the queue is empty.
fn main() {
    let queue = Mutex::new(VecDeque::new());


    println!("checkpoint 1 {:?}", thread::current().id());
    let timeout = Instant::now();
    thread::scope(|s| {
        // Consuming thread
        let t = s.spawn(
            {    
            || loop {
                dbg!(timeout.elapsed());
                let item = queue.lock().unwrap().pop_front();
                if let Some(item) = item {
                    dbg!(item);
                } else {
                    thread::park();
                }
            }    
        });

        println!("checkpoint 2 {:?}", thread::current().id());
        // Producing thread
        for i in 0.. {
            queue.lock().unwrap().push_back(i);
            t.thread().unpark();
            thread::sleep(std::time::Duration::from_secs(10));
        }
    });
}