use std::sync::{Arc, Mutex, RwLock};

use ::tokio::spawn;
use blox_assets::AssetManager;
use blox_interpreter::{load_module_from_string, Scope};
use tracing::info;

pub struct BloxEnvironment {
    assets: Arc<Mutex<AssetManager>>,
    scope: Arc<RwLock<Scope>>,
}

impl BloxEnvironment {
    pub fn new(assets: Arc<Mutex<AssetManager>>) -> Self {
        let scope = create_scope(assets.clone());
        let scope = Arc::new(RwLock::new(scope));

        BloxEnvironment { assets, scope }
    }

    pub fn start(&self) {
        let on_change = self.assets.lock().unwrap().on_change();
        let assets = self.assets.clone();
        let scope = self.scope.clone();

        spawn(async move {
            loop {
                on_change.notified().await;
                info!("Assets changed");

                let mut scope = scope.write().unwrap();
                *scope = create_scope(assets.clone());
            }
        });
    }
}

const STDLIB: [(&'static str, &'static str); 1] =
    [("stdlib/math.blox", include_str!("../../stdlib/math.blox"))];

pub(crate) fn create_scope(assets: Arc<Mutex<AssetManager>>) -> Scope {
    let scope = Scope::default();

    // load the standard library
    for (path, source) in STDLIB.iter() {
        let module =
            load_module_from_string(path, source).expect("failed to load stdlib module {path}");

        module.exports.iter().for_each(|(name, value)| {
            scope.insert_binding(name, value.clone());
        });
    }

    scope
}
