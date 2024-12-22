use blox_language::ast;

use crate::{expression::evaluate_expression, EvaluationContext, RuntimeError, Value};

use super::assign_to_expression;

pub fn assign_to_object_index(
    object_index: &ast::ObjectIndex,
    value: Value,
    context: &mut EvaluationContext,
) -> Result<(), RuntimeError> {
    let ast::ObjectIndex { base, index } = object_index;
    let base_value = evaluate_expression(&base, context)?;

    match base_value {
        Value::Object(mut members) => {
            members.insert(index.name.clone(), value);
            assign_to_expression(&base, Value::Object(members), context)
        }
        base_value => Err(RuntimeError::NotAnObject {
            object_expression: *base.clone(),
            object_value: base_value.clone(),
            key: index.name.clone(),
        }),
    }
}
