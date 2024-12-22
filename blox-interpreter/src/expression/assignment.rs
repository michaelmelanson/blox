mod array_index;
mod identifier;
mod object_index;

use array_index::assign_to_array_index;
use blox_language::ast;
use identifier::assign_to_identifier;
use object_index::assign_to_object_index;
use tracing::Level;

use crate::{EvaluationContext, RuntimeError, Value};

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn assign_to_expression(
    target: &ast::Expression,
    value: Value,
    context: &mut EvaluationContext,
) -> Result<(), RuntimeError> {
    match target {
        ast::Expression::Term(ast::ExpressionTerm::Identifier(identifier)) => {
            assign_to_identifier(identifier, value, context)
        }
        ast::Expression::Term(ast::ExpressionTerm::ArrayIndex(array_index)) => {
            assign_to_array_index(array_index, value, context)
        }
        ast::Expression::Term(ast::ExpressionTerm::ObjectIndex(object_index)) => {
            assign_to_object_index(object_index, value, context)
        }

        target => Err(RuntimeError::LhsNotAssignable {
            expression: target.clone(),
            value: value.clone(),
        }),
    }
}
