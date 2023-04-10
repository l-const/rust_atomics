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
        self.num_waiters.fetch_add(1, Relaxed);
        let counter_value = self.counter.load(Relaxed);

        // Unlock the mutex so we can lock it again later.
        // but remember the mutex so we can lock it again later.
        let mutex = guard.mutex;
        drop(guard);
        
        // Wait, but only if the counter hasn't changed since unlocking.
        wait(&self.counter, counter_value);

        self.num_waiters.fetch_sub(1, Relaxed);

        mutex.lock()
    }

    pub fn notify_one(&self) {
        if self.num_waiters.load(Relaxed) > 0 {
            self.counter.fetch_add(1, Relaxed);
            wake_one(&self.counter);
        }
    }

    pub fn notify_all(& self) {
        if self.num_waiters.load(Relaxed) > 0 {
            self.counter.fetch_add(1, Relaxed);
            wake_all(&self.counter);
        }
    }
}


mod tests {
    use std::thread;
    use super::CondVar;


    #[test]
    fn test_condvar() {
        let mutex = crate::mutex_content::Mutex::new(0);
        let condvar = CondVar::new();

        let mut wakeups = 0;

        thread::scope(|s| {
            s.spawn(|| {
                thread::sleep(std::time::Duration::from_secs(1));
                *mutex.lock() = 123;
                condvar.notify_one();
            });


            let mut m  = mutex.lock();
            while *m < 100 {
                m = condvar.wait(m);
                wakeups += 1;
            }

            assert_eq!(*m, 123);
        });

    // Chceck that the main thread actually did wait (not busy-loop),
    // while still allowing for a few spurious wak ups.
    assert!(wakeups < 10);
    assert!(wakeups == 1);
    }

}