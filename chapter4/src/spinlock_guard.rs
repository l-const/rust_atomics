use std::{sync::atomic::AtomicBool};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

pub struct Guard<'a, T> {
    lock:  &'a SpinLock<T>,
}

impl <'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, std::sync::atomic::Ordering::Release);
    }
}

impl <'a, T> Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe{ &*self.lock.value.get() }
    }
}

impl <'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
     // Safety: The very existence of this Guard
        // guarenteeeds we've exclusively locked the lock.
        unsafe{ &mut *self.lock.value.get() }   
    }
}

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl <T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self)  -> Guard<T> {
        while self.locked.swap(true, std::sync::atomic::Ordering::Acquire) {
            std::hint::spin_loop();
        }

        Guard{
            lock: self,
        }
       
    }

    pub fn unlock(guard: Guard<T>) {
        drop(guard)
    }
}   

unsafe impl<T> Sync for SpinLock<T> where T: Send {}