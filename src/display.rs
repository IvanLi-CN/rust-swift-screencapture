use core::fmt;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use log::{debug, info, warn};
use tokio::sync::{watch, OnceCell, RwLock};

use crate::ffi::{self, start_record};

pub type CGDisplayId = u32;

#[derive(Clone)]
pub struct Frame {
    pub bytes_per_row: isize,
    pub width: isize,
    pub height: isize,
    pub bytes: Arc<Vec<u8>>,
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame: {}x{}, len: {}, bytes_per_row: {}",
            self.width,
            self.height,
            self.bytes.len(),
            self.bytes_per_row
        )
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame: {}x{}, len: {}",
            self.width,
            self.height,
            self.bytes.len()
        )
    }
}

#[derive(Clone, Debug)]
pub struct Display {
    display_id: CGDisplayId,
}

impl Display {
    pub fn new(display_id: CGDisplayId) -> Self {
        Self { display_id }
    }

    pub async fn start_capture(&self) {
        let manager = DisplayManager::global().await;

        manager.start_capture(self.display_id).await;
    }

    pub async fn stop_capture(&self) {
        let manager = DisplayManager::global().await;

        manager.stop_capture(self.display_id).await;
    }

    pub async fn subscribe_frame(&self) -> watch::Receiver<Frame> {
        let (tx, rx) = watch::channel(Frame {
            bytes_per_row: 0,
            width: 0,
            height: 0,
            bytes: Arc::new(Vec::new()),
        });

        let display_id = self.display_id;

        tokio::spawn(async move {
            let manager = DisplayManager::global().await;
            let mut raw_rx = manager.all_frame_tx.subscribe();

            while raw_rx.changed().await.is_ok() {
                let (id, frame) = raw_rx.borrow().clone();

                if id != display_id {
                    continue;
                }

                if let Err(err) = tx.send(frame) {
                    warn!("frame send error: {}", err);
                }
                tokio::task::yield_now().await;
            }
        });

        return rx;
    }
}

pub struct DisplayManager {
    all_frame_tx: watch::Sender<(CGDisplayId, Frame)>,
    capturing_display_ids: Arc<RwLock<HashMap<CGDisplayId, usize>>>,
}

impl DisplayManager {
    fn new() -> Self {
        set_handle(tokio::runtime::Handle::current());
        let (all_frame_tx, _) = watch::channel((
            0,
            Frame {
                bytes_per_row: 0,
                width: 0,
                height: 0,
                bytes: Arc::new(Vec::new()),
            },
        ));
        Self {
            all_frame_tx,
            capturing_display_ids: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn global() -> &'static Self {
        static INSTANCE: OnceCell<DisplayManager> = OnceCell::const_new();
        INSTANCE
            .get_or_init(|| async { DisplayManager::new() })
            .await
    }

    pub async fn frame(&self, display_id: CGDisplayId, frame: Frame) {
        match self.all_frame_tx.send((display_id, frame)) {
            Ok(_) => {
                debug!("display#{} frame sent to all_frame_tx", display_id);
            }
            Err(err) => {
                warn!(
                    "display#{} frame sent to all_frame_tx failed: {}",
                    display_id, err
                );
            }
        }
        tokio::task::yield_now().await;
    }

    pub async fn start_capture(&self, display_id: CGDisplayId) {
        let mut ids = self.capturing_display_ids.write().await;

        if let Some(count) = ids.get_mut(&display_id) {
            *count += 1;
            return;
        }

        start_record(display_id);
        ids.insert(display_id, 1);
    }

    pub async fn stop_capture(&self, display_id: CGDisplayId) {
        let mut ids = self.capturing_display_ids.write().await;

        if let Some(count) = ids.get_mut(&display_id) {
            *count -= 1;
            if *count == 0 {
                ids.remove(&display_id);
                ffi::stop_record();
            }
            return;
        }

        ffi::stop_record()
    }
}

pub static HANDLE: OnceCell<Mutex<Option<tokio::runtime::Handle>>> = OnceCell::const_new();

pub fn set_handle(handle: tokio::runtime::Handle) {
    HANDLE.set(Mutex::new(Some(handle))).unwrap();
}
