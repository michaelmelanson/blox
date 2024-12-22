use blox_language::ast::Identifier;

use crate::{EvaluationContext, RuntimeError, Value};

pub fn assign_to_identifier(
    identifier: &Identifier,
    value: Value,
    context: &mut EvaluationContext,
) -> Result<(), RuntimeError> {
    context.scope.insert_binding(&identifier, value);
    Ok(())
}
