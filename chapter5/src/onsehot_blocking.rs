use std::marker::PhantomData;
use std::sync::atomic::Ordering::*;
use std::thread::{self, Thread};
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};

struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    pub fn split<'b>(&'b mut self) -> (Sender<'b, T>, Receiver<'b, T>) {
        *self = Self::new();
        (
            Sender {
                channel: self,
                receiving_thread: std::thread::current(),
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            },
        )
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { (*self.message.get()).assume_init_drop() }
        }
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiving_thread: Thread,
}

impl<'a, T> Sender<'a, T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Release);
        self.receiving_thread.unpark();
    }
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    _no_send: PhantomData<*const ()>,
}

impl<'a, T> Receiver<'a, T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    pub fn receive(self) -> T {
        while !self.channel.ready.swap(false, Acquire) {
            thread::park();
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

mod tests {

    use super::*;
    use std::thread;

    #[test]
    fn test_first() {
        let mut channel = Channel::new();
        thread::scope(|s| {
            let (sender, receiver) = channel.split();
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
