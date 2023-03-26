use std::{sync::atomic::AtomicBool};
use std::cell::UnsafeCell;


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

    pub fn lock(&self)  -> &mut T {
        while self.locked.swap(true, std::sync::atomic::Ordering::Acquire) {
            std::hint::spin_loop();
        }
        // Safety: managed to lock and get exclusice access
        unsafe { &mut *self.value.get() }
    }

    /// Safety: The &mut T from lock() must be gone!
    /// (And no cheating by keeping reference to fields of that T around!)
    pub unsafe fn unlock(&mut self) {
        self.locked.store(false, std::sync::atomic::Ordering::Release);
    }
}   

unsafe impl<T> Sync for SpinLock<T> where T: Send {}