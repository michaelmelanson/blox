use blox_language::ast;

use crate::{expression::evaluate_expression, Scope};

pub fn execute_statement(statement: &ast::Statement, scope: &mut Scope) {
    match statement {
        ast::Statement::Binding { lhs, rhs } => {
            if let Some(value) = evaluate_expression(rhs, &scope) {
                scope.bindings.insert(lhs.clone(), value);
            } else {
                unimplemented!();
            }
        }

        ast::Statement::FunctionCall(_call) => {
            unimplemented!()
        }
    }
}
