use std::sync::Arc;

use display::CGDisplayId;
use log::{debug, warn};

use crate::display::{DisplayManager, HANDLE, Frame};

pub mod display;

#[swift_bridge::bridge]
pub mod ffi {

    extern "Rust" {
        fn frame(id: u32, bytes_per_row: isize, width: isize, height: isize, bytes: &[u8]);
        fn stopped(id: u32);
    }

    extern "Swift" {
        fn start_record(displayId: u32, frameRate: i32);
        fn stop_record();
    }
}

fn frame(id: CGDisplayId, bytes_per_row: isize, width: isize, height: isize, bytes: &[u8]) {
    // debug!("frame: {}x{}. bytes_per_row: {}", width, height, bytes_per_row);

    let bytes = bytes.to_vec();

    let handle = HANDLE.get().unwrap().clone();
    let handle = handle.lock().unwrap().clone().unwrap();

    handle.spawn(async move {
        DisplayManager::global().await.frame(id, Frame {
            bytes_per_row,
            width,
            height,
            bytes: Arc::new(bytes),
        }).await;
    });
}

fn stopped(id: CGDisplayId) {
    debug!("stopped: {}", id);

    let handle = HANDLE.get().unwrap().clone();
    let handle = handle.lock().unwrap().clone().unwrap();

    handle.spawn(async move {
        DisplayManager::global().await.stopped(id).await;
    });
}
