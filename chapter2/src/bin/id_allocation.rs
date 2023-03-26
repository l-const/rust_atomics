use std::sync::atomic::AtomicU32;

fn main() {
    allocate_new_id();
    allocate_new_id();
    allocate_new_id();
}


fn allocate_new_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    dbg!(NEXT_ID.load(std::sync::atomic::Ordering::Relaxed))
}