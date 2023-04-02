use std::sync::atomic::Ordering::*;
use std::sync::Arc;
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};

struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    pub(crate) const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { (*self.message.get()).assume_init_drop() }
        }
    }
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Release);
    }
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Receiver<T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    pub fn receive(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            panic!("message is not ready");
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let c = Channel::new();
    let arc = Arc::new(c);
    let s = Sender {
        channel: Arc::clone(&arc),
    };
    let r = Receiver { channel: arc };
    (s, r)
}

mod tests {

    use super::*;
    use std::thread;

    #[test]
    fn test_first() {
        thread::scope(|s| {
            let (sender, receiver) = channel();
            let t = thread::current();
            s.spawn(move || {
                sender.send("hello world");
                t.unpark();
            });

            while !receiver.is_ready() {
                thread::park();
            }
            assert_eq!(receiver.receive(), "hello world");
        })
    }
}
