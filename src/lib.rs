use display::CGDisplayId;
use log::debug;
use tokio::runtime::Handle;

use crate::display::{DisplayManager, HANDLE};

pub mod display;

#[swift_bridge::bridge]
pub mod ffi {

    extern "Rust" {
        fn frame(id: u32, width: isize, height: isize, bytes: &[u8]);
    }

    extern "Swift" {
        fn start_record(displayId: u32);
        fn stop_record();
    }
}

fn frame(id: CGDisplayId, width: isize, height: isize, bytes: &[u8]) {
    debug!("frame: {}x{}", width, height);

    let bytes = bytes.to_vec();

    let handle = HANDLE.get().unwrap().clone();
    let handle = handle.lock().unwrap().clone().unwrap();

    handle.spawn(async move {
        DisplayManager::global().await.frame(id, width, height, bytes).await;
    });
}
