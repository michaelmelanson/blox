use blox_language::ast;
use tracing::Level;

use crate::{EvaluationContext, RuntimeError, Value};

use super::evaluate_expression;

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn evaluate_condition(
    expression: &ast::Expression,
    context: &mut EvaluationContext,
) -> Result<bool, RuntimeError> {
    let condition_value = evaluate_expression(expression, context)?;

    let is_truthy = match condition_value {
        Value::Boolean(value) => value,
        Value::Number(number) => number.is_sign_positive() && !number.is_zero(),
        condition_value => {
            return Err(RuntimeError::InvalidCondition {
                condition_expression: expression.clone(),
                condition_value,
            });
        }
    };

    Ok(is_truthy)
}
