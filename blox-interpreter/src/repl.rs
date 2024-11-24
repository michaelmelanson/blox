use std::sync::Arc;

use blox_language::Parser;
use rustyline::error::ReadlineError;

use crate::{execute_program, Scope};

#[derive(Debug)]
pub enum BloxReplError {
    ReadlineError(ReadlineError),
}

impl std::fmt::Display for BloxReplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BloxReplError::ReadlineError(e) => write!(f, "Readline error: {e}"),
        }
    }
}

impl From<ReadlineError> for BloxReplError {
    fn from(error: ReadlineError) -> Self {
        BloxReplError::ReadlineError(error)
    }
}

impl std::error::Error for BloxReplError {}

pub fn start_repl(mut scope: Arc<Scope>) -> Result<(), BloxReplError> {
    let mut editor = rustyline::DefaultEditor::new()?;
    editor.load_history(".blox-history")?;

    'repl: loop {
        let line = editor.readline("blox> ");

        match line {
            Ok(line) => {
                editor.add_history_entry(&line)?;

                let parser = Parser::new(&line);

                match parser.parse() {
                    Ok(ast) => {
                        let value = execute_program(&ast, &mut scope);
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
    Ok(())
}