use std::sync::Arc;

use blox_interpreter::{start_repl, BloxReplError, Scope};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct BloxArgs {
    file: Option<String>,
}

pub fn main() -> Result<(), BloxReplError> {
    let args = BloxArgs::parse();

    if let Some(file) = args.file {
        println!("File: {}", file);
    } else {
        let scope = Arc::new(Scope::default());
        start_repl(scope)?;
    }

    Ok(())
}
