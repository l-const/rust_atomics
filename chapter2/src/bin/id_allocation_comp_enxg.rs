use std::sync::atomic::{AtomicU32, self};
use std::sync::atomic::Ordering::Relaxed;

fn main() {
    dbg!(allocate_new_id());
    dbg!(allocate_new_id());
    dbg!(allocate_new_id());
}


fn allocate_new_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    let mut id = NEXT_ID.load(atomic::Ordering::Relaxed);
    loop {
        assert!(id < 1000, "too many ids!");
        match NEXT_ID.compare_exchange(id, id + 1, Relaxed, Relaxed) {
            Ok(id) => return id,
            Err(v) => id = v,
        }
    }
}