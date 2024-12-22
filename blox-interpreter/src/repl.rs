use blox_language::parser::Parser;
use rustyline::error::ReadlineError;

use crate::{execute_program, module::EvaluationContext};

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

pub fn start_repl(mut context: EvaluationContext) -> Result<(), BloxReplError> {
    let mut editor = rustyline::DefaultEditor::new()?;
    let _ = editor.load_history(".blox-history");

    'repl: loop {
        let line = editor.readline("blox> ");

        match line {
            Ok(line) => {
                editor.add_history_entry(&line)?;

                let parser = Parser::new("<repl>", &line);

                match parser.parse() {
                    Ok(ast) => {
                        let value = execute_program(&ast, &mut context);
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
