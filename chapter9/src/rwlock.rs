use std::{cell::UnsafeCell, sync::atomic::AtomicU32, ops::{Deref, DerefMut}};
use std::sync::atomic::Ordering::*;

use atomic_wait::{wait, wake_one, wake_all};

pub struct RwLock<T> {
    /// The number of readers, or u32::MAX if wite-locked.
    state: AtomicU32,
    writer_wake_counter: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send + Sync> Sync for RwLock<T> {}


impl <T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0), // Unlocked
            writer_wake_counter: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn read(&self) -> ReadGuard<T> {
        let mut s = self.state.load(Relaxed);
        loop {
            if s < u32::MAX {
                assert!(s != u32::MAX - 1, "too many readers");
                match self.state.compare_exchange_weak(s, s + 1, Acquire, Relaxed) {
                    Ok(_) => return ReadGuard { rwlock: self},
                    Err(e) => s = e,
                }
            }
            if s == u32::MAX {
                wait(&self.state, u32::MAX);
                s = self.state.load(Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuard<T> {
        while self.state.compare_exchange(0, u32::MAX, Acquire, Relaxed).is_err() {
            let w = self.writer_wake_counter.load(Acquire);
            if self.state.load(Relaxed) != 0 {
                // Wait while already locked.
                wait(&self.writer_wake_counter, w);
            }
        }
        WriteGuard { rwlock: self }
    }

}


pub struct ReadGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl <T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.rwlock.state.fetch_sub(1, Release) == 1 {
            self.rwlock.writer_wake_counter.fetch_add(1, Release);
            wake_one(&self.rwlock.writer_wake_counter);
        }
    }
}

impl<T>  std::ops::Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get()}
    }
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl <T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.state.store(0, Release);  
        // Wake up all waiting readers and writers
        wake_all(&self.rwlock.state);
    }
}

impl<T>  Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get()}
    }
}

impl <T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.rwlock.value.get()}
    }
}