
use std::{sync::atomic::{AtomicUsize, fence}, ptr::NonNull, ops::Deref, cell::UnsafeCell};
use std::sync::atomic::Ordering::*;

struct ArcData<T> {
    /// Number of `Arc`s
    data_ref_count: AtomicUsize,
    /// Number of `Arc`s and `Weak`s combined.
    alloc_ref_count: AtomicUsize,
    // The data. `None` if theres only weak pointers left.
    data: UnsafeCell<Option<T>>,
}


pub struct Arc<T> {
    weak: Weak<T>,
}


pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>
}

impl <T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }
}

unsafe impl<T: Sync + Send> Send for Weak<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Arc {
            weak: Weak {
                ptr: NonNull::from(Box::leak(Box::new(ArcData {
                    alloc_ref_count: AtomicUsize::new(1),
                    data_ref_count: AtomicUsize::new(1),
                    data: UnsafeCell::new(Some(data)),
                }))),
            }
        }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.weak.data().alloc_ref_count.load(Relaxed) == 1 {
            fence(std::sync::atomic::Ordering::Acquire);
            // Safety: Nothing else can access the data, since
            // there's only one Arc, to which we have exclusive access.
            unsafe { Some(&mut arc.ptr.as_mut().data) } 
        } else {
            None
        }
    }
}

impl <T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let ptr = self.weak.data().data.get();
         // Safety: Since there's an Arc to the data,
        // the data exists and may be shared.
        unsafe {(*ptr).as_ref().unwrap()}
    }
}

impl <T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        // TODO: Handle overflows.
        if self.data().data_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Weak { ptr: self.ptr }
    }
}

impl <T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let weak = self.weak.clone();
        if weak.data().data_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Arc { weak }
    }
}


impl <T> Drop for Arc<T> {
    fn drop(&mut self) {
        // TODO: Memory ordering
        if self.weak.data().data_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            let ptr = self.weak.data().data.get();
             // Safety: The data reference counter is zero,
            // so nothing will access it.
            unsafe {
                *ptr = None;
            }
        }
    }
}

fn hello() {
 todo!()
}


mod tests {
    use std::sync::atomic::AtomicUsize;


    #[test]
    fn test() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

        struct DetectDrop;

        impl Drop for DetectDrop {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        // Create two Arcs sharing an object containing a string
        // and a DetectDrop, to detect when its dropped.
        let x = std::sync::Arc::new(("Hello", DetectDrop));
        let y = x.clone();

        // Send x to another thread, and use it there
        let t = std::thread::spawn(move || {
            assert_eq!(x.0, "Hello");
        });

        // In parallel, y should still be usable here.
        assert_eq!(y.0, "Hello");


        // Wait for the thread to finish 
        t.join().unwrap();

        // On Arc, x, should be dropped by now.
        // We still have y. so the object shouldn't have been dropped yet.
        assert_eq!(NUM_DROPS.load(std::sync::atomic::Ordering::Relaxed), 0);

        // Drop the remaining `Arc`
        drop(y);

        // Now that y is dropped too,
        // the object should've been dropped.
        assert_eq!(NUM_DROPS.load(std::sync::atomic::Ordering::Relaxed), 1);
    }    
}