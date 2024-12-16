use std::sync::{Arc, Mutex, RwLock};

use ::tokio::spawn;
use blox_assets::AssetManager;
use blox_interpreter::{load_module_from_string, EvaluationContext, Scope};
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

const STDLIB: [(&'static str, &'static str); 3] = [
    ("stdlib/math.blox", include_str!("../../stdlib/math.blox")),
    ("stdlib/list.blox", include_str!("../../stdlib/list.blox")),
    (
        "stdlib/database.blox",
        include_str!("../../stdlib/database.blox"),
    ),
];

pub(crate) fn create_context(assets: Arc<Mutex<AssetManager>>) -> EvaluationContext {
    let asset_manager = assets.lock().unwrap();
    let base_dir = asset_manager
        .base_dir()
        .to_str()
        .expect("could not convert asset base dir to string");

    let context = EvaluationContext::new(base_dir, &Arc::new(Scope::default()));

    // load the standard library
    for (path, source) in STDLIB.iter() {
        let module = load_module_from_string(path, source, &context)
            .expect("failed to load stdlib module {path}");

        module.exports.iter().for_each(|(name, value)| {
            context.scope.insert_binding(name, value.clone());
        });
    }

    context
}
