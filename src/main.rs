use std::{thread, time::Duration};

use log::info;
use rust_swift_screencapture::display::Display;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let display_id = args[1].parse::<u32>().unwrap();

    println!("display_id: {}", display_id);

    let display = Display::new(display_id);

    display.start_capture(30).await;

    let mut rx = display.subscribe_frame().await;
    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let frame = rx.borrow().clone();
            // info!("frame: {}", frame);
            tokio::task::yield_now().await;
        }
        println!("frame rx stopped")
    });

    println!("Press any key to exit...");

    // wait stdin to exit
    let mut input = String::new();
    while std::io::stdin().read_line(&mut input).is_err() {
        thread::sleep(Duration::from_millis(100));
    }
    println!("input: {}", input);
    display.stop_capture().await;
}
