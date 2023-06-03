use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{info, warn};
use tokio::sync::{OnceCell, RwLock};

use crate::ffi;

pub type CGDisplayId = u32;

#[derive(Clone)]
pub struct Display {
    display_id: CGDisplayId,
}

impl Display {
    pub fn new(display_id: CGDisplayId) -> Self {
        Self { display_id }
    }

    pub async fn start_capture(&self) {
        let manager = DisplayManager::global().await;

        manager.start_capture(&self).await;
    }

    pub async fn stop_capture(&self) {
        let manager = DisplayManager::global().await;

        manager.stop_capture(&self).await;
    }

    pub fn frame(&self, width: isize, height: isize, bytes: Vec<u8>) {
        info!(
            "frame received. size: {}x{}, bytes: {}",
            width,
            height,
            bytes.len()
        );
    }
}

pub struct DisplayManager {
    displays: Arc<RwLock<HashMap<CGDisplayId, Display>>>,
}

impl DisplayManager {
    fn new() -> Self {
        set_handle(tokio::runtime::Handle::current());
        Self {
            displays: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn global() -> &'static Self {
        static INSTANCE: OnceCell<DisplayManager> = OnceCell::const_new();
        INSTANCE
            .get_or_init(|| async { DisplayManager::new() })
            .await
    }

    pub async fn frame(
        &self,
        display_id: CGDisplayId,
        width: isize,
        height: isize,
        bytes: Vec<u8>,
    ) {
        let displays = self.displays.read().await;

        let display = displays.get(&display_id);

        if let Some(display) = display {
            display.frame(width, height, bytes);
        } else {
            warn!("display not includes: {}", display_id);
        }
    }

    pub async fn start_capture(&self, display: &Display) {
        let mut displays = self.displays.write().await;

        displays.insert(display.display_id, display.clone());

        ffi::start_record(display.display_id)
    }

    pub async fn stop_capture(&self, display: &Display) {
        let mut displays = self.displays.write().await;

        displays.remove(&display.display_id);

        ffi::stop_record()
    }

}

pub static HANDLE: OnceCell<Mutex<Option<tokio::runtime::Handle>>> = OnceCell::const_new();

pub fn set_handle(handle: tokio::runtime::Handle) {
    HANDLE.set(Mutex::new(Some(handle))).unwrap();
}

