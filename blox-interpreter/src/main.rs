use std::sync::Arc;

use blox_interpreter::Scope;
use blox_language::parse;
use clap::Parser;
use rustyline::error::ReadlineError;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct BloxArgs {
    file: Option<String>,
}

pub fn main() -> rustyline::Result<()> {
    let args = BloxArgs::parse();

    if let Some(file) = args.file {
        println!("File: {}", file);
    } else {
        let mut editor = rustyline::DefaultEditor::new()?;
        editor.load_history(".blox-history")?;

        let mut scope = Arc::new(Scope::default());

        'repl: loop {
            let line = editor.readline("blox> ");

            match line {
                Ok(line) => {
                    editor.add_history_entry(&line)?;

                    match parse(&line) {
                        Ok(ast) => {
                            let value = blox_interpreter::execute_program(&ast, &mut scope);
                            match value {
                                Ok(value) => println!("{}", value),
                                Err(e) => eprintln!("Error: {e}"),
                            }
                        }
                        Err(e) => eprintln!("Error: {e}"),
                    }
                }

                Err(ReadlineError::Interrupted) => break 'repl,
                Err(ReadlineError::Eof) => break 'repl,
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                }
            }
        }

        editor.save_history(".blox-history")?;
    }

    Ok(())
}
