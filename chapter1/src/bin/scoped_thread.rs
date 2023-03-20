fn main() {
    let mut numbers = vec![1, 4 , 7];

    std::thread::scope(|s| {
        let result = s.spawn(|| {
            println!("First {:?}", &std::thread::current().id());
            return 45 * numbers.len();
        });
        // println!("Scope println! {:?}", &result.join());
       let second =  s.spawn(|| {
            println!("First {:?}", &std::thread::current().id());
            return 90 *numbers.len();
        });
        // println!("Second Scope println! {:?}", &second.join());
    });
}
