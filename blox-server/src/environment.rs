use std::sync::{Arc, Mutex, RwLock};

use ::tokio::spawn;
use blox_assets::AssetManager;
use blox_interpreter::{load_stdlib, EvaluationContext, Scope};
use tracing::info;

pub struct BloxEnvironment {
    assets: Arc<Mutex<AssetManager>>,
    context: Arc<RwLock<EvaluationContext>>,
}

impl BloxEnvironment {
    pub fn new(assets: Arc<Mutex<AssetManager>>) -> Self {
        let context = create_context(assets.clone());
        let context = Arc::new(RwLock::new(context));
        BloxEnvironment { assets, context }
    }

    pub fn context(&self) -> Arc<RwLock<EvaluationContext>> {
        self.context.clone()
    }

    pub fn assets(&self) -> Arc<Mutex<AssetManager>> {
        self.assets.clone()
    }

    pub fn start(&self) {
        let on_change = self.assets.lock().unwrap().on_change();
        let assets = self.assets.clone();
        let context = self.context.clone();

        spawn(async move {
            loop {
                on_change.notified().await;
                info!("Assets changed");

                let mut context = context.write().unwrap();
                *context = create_context(assets.clone());
            }
        });
    }
}

pub(crate) fn create_context(assets: Arc<Mutex<AssetManager>>) -> EvaluationContext {
    let asset_manager = assets.lock().unwrap();
    let base_dir = asset_manager
        .base_dir()
        .to_str()
        .expect("could not convert asset base dir to string");

    let import_cache = Arc::new(RwLock::new(Default::default()));

    let mut loader_context =
        EvaluationContext::new(base_dir, Arc::new(Scope::default()), import_cache.clone());

    load_stdlib(&mut loader_context);

    EvaluationContext::new(base_dir, Arc::new(Scope::default()), import_cache)
}
