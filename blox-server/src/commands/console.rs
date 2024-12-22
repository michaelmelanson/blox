use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use blox_assets::AssetManager;
use blox_interpreter::{start_repl, Intrinsic, Value};
use blox_language::ast::Identifier;
use tracing::info;

use crate::environment::create_context;

pub async fn console_command(directory: &str) -> Result<(), anyhow::Error> {
    let assets = AssetManager::new(&directory)?;
    let assets = Arc::new(Mutex::new(assets));

    let context = create_context(assets);

    context.scope.insert_binding(
        &Identifier {
            name: "print".to_string(),
        },
        Value::Intrinsic(Intrinsic::new(
            "print",
            Arc::new(|arguments: HashMap<Identifier, Value>| {
                if let Some(message) = arguments.get(&Identifier {
                    name: "message".to_string(),
                }) {
                    info!(message = message.to_string());
                }

                Ok(Value::Void)
            }),
        )),
    );

    start_repl(context)?;

    Ok(())
}
