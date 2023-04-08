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
    mutex: &'a Mutex<T>
}


impl <T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0), // unlocked state
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&mut self) -> MutexGuard<T> {
        // Set the state to 1: locked
        if self.state.compare_exchange(0, 1, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed).is_err() {
            while self.state.swap(2, std::sync::atomic::Ordering::Acquire) != 0 {
                wait(&self.state, 2);
            }
        }
        MutexGuard { mutex: self }
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