mod spinlock;
mod spinlock_unsf;
mod spinlock_guard;


use std::thread;

use spinlock_guard::SpinLock;

fn main() {
    let x = SpinLock::new(Vec::new());

    thread::scope(|s| {
        s.spawn(|| x.lock().push(1) );
        s.spawn(|| {
            let mut g = x.lock();
            g.push(2);
            g.push(2);
        });
    });
    let g  = x.lock();
    //SpinLock::unlock(g); drop guard
    assert!(g.as_slice() == [1,2,2] || g.as_slice() == [2,2,1]);
}
