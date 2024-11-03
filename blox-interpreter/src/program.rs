use blox_language::ast;

use crate::{statement::execute_statement, RuntimeError, Scope, Value};

pub fn execute_program(program: &ast::Program, scope: &mut Scope) -> Result<Value, RuntimeError> {
    let mut value = Value::Void;

    for statement in &program.block.statements {
        value = execute_statement(statement, scope)?;
    }

    Ok(value)
}
