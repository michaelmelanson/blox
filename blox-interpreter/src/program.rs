use std::convert::Infallible;

use blox_language::ast;

use crate::{statement::execute_statement, Scope};

pub fn execute_program(program: &ast::Program, scope: &mut Scope) -> Result<(), Infallible> {
    for statement in &program.block.statements {
        execute_statement(statement, scope);
    }

    Ok::<_, Infallible>(())
}
