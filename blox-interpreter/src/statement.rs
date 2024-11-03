use blox_language::ast;

use crate::{expression::evaluate_expression, RuntimeError, Scope, Value};

pub fn execute_statement(
    statement: &ast::Statement,
    scope: &mut Scope,
) -> Result<Value, RuntimeError> {
    match statement {
        ast::Statement::Expression(expression) => {
            let value = evaluate_expression(expression, scope)?;
            Ok(value)
        }
        ast::Statement::Binding { lhs, rhs } => {
            let value = evaluate_expression(rhs, &scope)?;
            scope.insert_binding(lhs, value.clone());
            Ok(value)
        }
    }
}
