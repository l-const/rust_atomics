use std::thread;

fn main() {
    let t1 = thread::spawn(callable);
    let t2 = thread::spawn(callable);
    let t3 = thread::Builder::new().name("obscure_thread".into()).spawn(callable).unwrap();
    let main_thread_id = thread::current().id();
    println!("Hello from the main thead with id:{main_thread_id:?}");
    t1.join();
    t2.join();
    t3.join();
    return_value_from_callable();
}

fn callable() {
    let id = thread::current().id();
    println!("Current thread id is: {id:?}");
}

fn return_value_from_callable() {
    
}
