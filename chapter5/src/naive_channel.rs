use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    pub fn receive(&self) -> T {
        let mut b = self.queue.lock().unwrap();
        loop {
            // Return if there is something already in the queue
            if let Some(message) = b.pop_front() {
                return message;
            }
            // wait/block until notified that something is ready to read
            b = self.item_ready.wait(b).unwrap();
        }
    }
}

mod tests {
    pub use super::*;

    #[test]
    fn test_channel() {
        let channel = Channel::new();
        std::thread::scope(|s| {
            s.spawn(|| {
                let received = channel.receive();
                dbg!("Received {}", received);
            });
            channel.send("hello");
            channel.send("world");
            channel.send("skata");
        });

        let last_received = channel.receive();
        dbg!("Received {}", last_received);
        assert_eq!(last_received, "world");
        assert_eq!(channel.receive(), "skata");
    }
}
