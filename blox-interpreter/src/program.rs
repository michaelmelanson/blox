use blox_language::ast;

use crate::{statement::execute_statement, RuntimeError, Scope, Value};

pub fn execute_program(program: &ast::Program, scope: &mut Scope) -> Result<Value, RuntimeError> {
    evaluate_block(&program.block, scope)
}

pub fn evaluate_block(block: &ast::Block, scope: &mut Scope) -> Result<Value, RuntimeError> {
    let mut value = Value::Void;

    for statement in &block.statements {
        value = execute_statement(statement, scope)?;
    }

    Ok(value)
}
