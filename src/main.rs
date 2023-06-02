use std::{thread, time::Duration};

use crate::ffi::stop_record;

fn main() {
    let start_num = 100;

    println!("The Rust starting number is {}.", start_num);

    let num = ffi::swift_multiply_by_4(start_num);

    println!("Printing the number from Rust...");
    println!("The number is now {}.", num);

    ffi::start_record();

    // wait stdin to exit
    let mut input = String::new();
    while std::io::stdin().read_line(&mut input).is_err() {
        thread::sleep(Duration::from_millis(100));
    }
    println!("input: {}", input);
    stop_record();

}

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        fn rust_double_number(num: i64) -> i64;
        fn frame(size: i64);
    }

    extern "Swift" {
        fn swift_multiply_by_4(num: i64) -> i64;
        fn start_record();
        fn stop_record();
    }
}

fn rust_double_number(num: i64) -> i64 {
    println!("Rust double function called...");

    num * 2
}

fn frame(size: i64) {
    println!("Rust frame function called. size: {}", size);
}
