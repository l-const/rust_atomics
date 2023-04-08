use std::sync::atomic::{AtomicU32, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;

use atomic_wait::{wake_one, wake_all, wait};

use crate::mutex_content::MutexGuard;

pub struct CondVar {
    counter: AtomicU32,
    num_waiters: AtomicUsize,
}


impl CondVar {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            num_waiters: AtomicUsize::new(0)
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let counter_value = self.counter.load(Relaxed);

        // Unlock the mutex so we can lock it again later.
        // but remember the mutex so we can lock it again later.
        let mutex = guard.mutex;
        drop(guard);
        
        // Wait, but only if the counter hasn't changed since unlocking.
        wait(&self.counter, counter_value);

        mutex.lock()
    }

    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_one(&self.counter);
    }

    pub fn notify_all(& self) {
        self.counter.fetch_add(1, Relaxed);
        wake_all(&self.counter);
    }
}