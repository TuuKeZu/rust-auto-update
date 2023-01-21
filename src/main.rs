const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Hello.rs v{VERSION}");
    println!("-------------------");
    println!("> Hello, world!");


    // terminal shouldn't terminate on close
    println!("Press any key to continue...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
