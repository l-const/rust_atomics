use std::{cell::UnsafeCell, sync::atomic::AtomicU32, ops::{Deref, DerefMut}};

use atomic_wait::wait;

pub struct  Mutex<T> {
    ///  0 : unlocked
    ///  1: locked
    /// 
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
        while self.state.swap(1, std::sync::atomic::Ordering::Acquire) == 1 {
            // If it was already locked..
            // .. wait, unless the state is no longer 1.
            wait(&self.state, 1)
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