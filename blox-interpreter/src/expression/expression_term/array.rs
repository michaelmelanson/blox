use blox_language::ast;

use crate::{expression::evaluate_expression, EvaluationContext, RuntimeError, Value};

pub fn evaluate_array(
    array: &ast::Array,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let mut members = Vec::new();
    for member_expression in array.0.iter() {
        let value = evaluate_expression(member_expression, context)?;
        members.push(value);
    }
    Ok(Value::Array(members))
}
