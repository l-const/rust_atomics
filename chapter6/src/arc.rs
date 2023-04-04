use std::{sync::atomic::{AtomicUsize, fence}, ptr::NonNull, ops::Deref};


struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>
}

impl<T>  Arc<T> {
    pub fn new(data: T) -> Self {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                ref_count: AtomicUsize::new(1),
                data,
            })))
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().ref_count.load(std::sync::atomic::Ordering::Relaxed) == 1 {
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
        &self.data().data
    }
}

impl <T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // TODO: Handle overflows.
        if self.data().ref_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Arc { ptr: self.ptr }
    }
}


impl <T> Drop for Arc<T> {
    fn drop(&mut self) {
        // TODO: Memory ordering
        if self.data().ref_count.fetch_sub(1, std::sync::atomic::Ordering::Release) == 1 {
            fence(std::sync::atomic::Ordering::Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
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