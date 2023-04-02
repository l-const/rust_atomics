use std::sync::atomic::Ordering::*;
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};
pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    in_use: AtomicBool,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    // A new channel is empty, with ready set to false,
    // and message left uninitialized
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            in_use: AtomicBool::new(false),
            ready: AtomicBool::new(false),
        }
    }
    // Safety: Only call this once!
    pub fn send(&self, message: T) {
        if self.in_use.swap(true, Relaxed) {
            panic!("Already in ue!")
        }
        unsafe { (*self.message.get()).write(message) };
        self.ready.store(true, Release);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Relaxed)
    }

    /// Panics if not message is available yet.
    /// Tip: Use `is_ready` to check first.
    pub fn receive(&self) -> T {
        if !self.ready.swap(false, Acquire) {
            panic!("no message available!");
        }
        // Safety: we've just checked (and reset) the ready flag.
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

mod tests {
    pub use super::Channel;
    use std::thread;

    #[test]
    fn test_first() {
        let channel = Channel::new();
        assert!(!channel.is_ready());
        let t = thread::current();
        thread::scope(|s| {
            s.spawn(|| {
                channel.send("ssasa");
                t.unpark();
            });

            while !channel.is_ready() {
                thread::park();
            }

            let received = channel.receive();
            assert_eq!(received, "ssasa");
        });
    }

    #[should_panic]
    #[test]
    fn test_receive_should_panic_empty_channel() {
        let channel: Channel<&str> = Channel::new();
        channel.receive();
    }
}
