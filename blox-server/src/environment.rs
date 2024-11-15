use std::sync::{Arc, Mutex};

use ::tokio::spawn;
use blox_assets::AssetManager;
use tracing::info;

pub struct BloxEnvironment {
    assets: Arc<Mutex<AssetManager>>,
}

impl BloxEnvironment {
    pub fn new(assets: Arc<Mutex<AssetManager>>) -> Self {
        BloxEnvironment {
            assets: assets.clone(),
        }
    }

    pub fn start(&self) {
        let on_change = self.assets.lock().unwrap().on_change();
        spawn(async move {
            loop {
                on_change.notified().await;
                info!("Assets changed");
            }
        });
    }
}
