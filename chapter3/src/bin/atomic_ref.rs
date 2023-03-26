use std::sync::atomic::{AtomicPtr, Ordering};


#[derive(Debug)]
struct Data;


fn get_data() -> &'static Data {
    static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());    

    let mut p = PTR.load(Ordering::Acquire);

    if p.is_null() {
      p = Box::into_raw(Box::new(genereate_data()));
      if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Ordering::Release,  Ordering::Acquire) {
        // Safety: p comes from Box::into_raw right above.
        // and want shared with another thread.
        drop(unsafe{ Box::from_raw(p)});
        p = e;
      }
    }
    // Safety: p is not null and points to a properly intialized value.
    unsafe {&*p}
}

fn genereate_data() -> Data {
    Data
}


fn main()  {
    get_data();
}