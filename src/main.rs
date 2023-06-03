use std::{thread, time::Duration};

use rust_binary_calls_swift_package::display::Display;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let display_id = args[1].parse::<u32>().unwrap();

    println!("display_id: {}", display_id);

    let display = Display::new(display_id);

    display.start_capture().await;

    println!("Press any key to exit...");

    // wait stdin to exit
    let mut input = String::new();
    while std::io::stdin().read_line(&mut input).is_err() {
        thread::sleep(Duration::from_millis(100));
    }
    println!("input: {}", input);
    display.stop_capture().await;
}
