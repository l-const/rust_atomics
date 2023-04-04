use std::{sync::atomic::AtomicUsize, ptr::NonNull, ops::Deref};


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
}

impl <T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data().data
    }
}


fn hello() {
 todo!()
}


mod tests {

    #[test]
    fn test_first() {

    }    
}