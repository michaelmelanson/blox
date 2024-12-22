use blox_language::ast;

use crate::{expression::evaluate_expression, EvaluationContext, RuntimeError, Value};

pub fn evaluate_object_index(
    object_index: &ast::ObjectIndex,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let ast::ObjectIndex { base, index } = object_index;
    let object_value = evaluate_expression(base, context)?;

    match object_value {
        Value::Object(ref members) => {
            if let Some(value) = members.get(&index.name) {
                Ok(value.clone())
            } else {
                Err(RuntimeError::ObjectKeyNotFound {
                    object_expression: *base.clone(),
                    object_value: object_value.clone(),
                    key: index.name.clone(),
                })
            }
        }
        object_value => Err(RuntimeError::NotAnObject {
            object_expression: *base.clone(),
            object_value: object_value.clone(),
            key: index.name.clone(),
        }),
    }
}
