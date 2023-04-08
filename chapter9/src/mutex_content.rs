use std::{cell::UnsafeCell, sync::atomic::AtomicU32, ops::{Deref, DerefMut}};

use atomic_wait::{wait, wake_one};

pub struct  Mutex<T> {
    ///  0 : unlocked
    ///  1: locked, no other threads waiting
    ///  2: locked, other threads waiting
    state: AtomicU32,
    value: UnsafeCell<T>
}

unsafe impl <T> Sync for Mutex<T> where T: Send {}


pub struct MutexGuard<'a, T> {
    pub(crate) mutex: &'a Mutex<T>
}


impl <T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0), // unlocked state
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        // Set the state to 1: locked
        if self.state.compare_exchange(0, 1, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed).is_err() {
          lock_contented(&self.state)
        }
        MutexGuard { mutex: self }
    }
}


fn lock_contented(state: &AtomicU32) {
    let mut spin_count = 0;

    while state.load(std::sync::atomic::Ordering::Relaxed) == 1 && spin_count < 100 {
        spin_count += 1;
        std::hint::spin_loop();
    }

    if state.compare_exchange(0, 1, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed).is_ok() {
        return;
    }

    while state.swap(2, std::sync::atomic::Ordering::Acquire) != 0 {
        wait(state, 2);
    }

}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get()}
    }
    
}


impl <T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.mutex.value.get()
        }
    }
}

impl <T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Set the state back to  0: unlocked

        if self.mutex.state.swap(0, std::sync::atomic::Ordering::Release) == 2 {
            wake_one(&self.mutex.state);
        }
    }
}


mod tests {
    use std::time::Instant;

    use super::*;
    #[test]
     fn test() {
        let m = Mutex::new(0);
        std::hint::black_box(&m);
        let start  = Instant::now();
        for _ in 0..5_000_000 {
            *m.lock() += 1;
        }
        let duration = start.elapsed();
        println!("locked {} times in  {:?}", *m.lock(), duration);
     }

     #[test]
     fn test_contention() {
        let m = Mutex::new(0);
        std::hint::black_box(&m);
        let start  = Instant::now();
        std::thread::scope(|s|{
            for _ in 0..4 {
                s.spawn(|| {
                    for _ in 0..5_000_000 {
                        *m.lock() += 1;
                    }
                });
            }
        });
        let duration = start.elapsed();
        println!("locked {} times in  {:?}", *m.lock(), duration);
     }
}