use blox_language::ast;

use crate::{expression::evaluate_expression, EvaluationContext, RuntimeError, Value};

pub fn evaluate_array_index(
    array_index: &ast::ArrayIndex,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let ast::ArrayIndex { base, index } = array_index;
    let array_value = evaluate_expression(base, context)?;
    let index_value = evaluate_expression(index, context)?;

    match (&array_value, &index_value) {
        (Value::Array(ref members), Value::Number(idx)) => {
            let Ok(idx): rust_decimal::Result<usize> = (*idx).try_into() else {
                return Err(RuntimeError::InvalidArrayIndex {
                    array_expression: *base.clone(),
                    array_value: array_value.clone(),
                    index_expression: *index.clone(),
                    index_value: index_value.clone(),
                });
            };

            if idx < members.len() {
                Ok(members[idx].clone())
            } else {
                Err(RuntimeError::ArrayIndexOutOfBounds {
                    array_expression: *base.clone(),
                    array_value: array_value.clone(),
                    index_expression: *index.clone(),
                    index_value: index_value.clone(),
                })
            }
        }
        (array_value, index_value) => Err(RuntimeError::InvalidArrayIndex {
            array_expression: *base.clone(),
            array_value: array_value.clone(),
            index_expression: *index.clone(),
            index_value: index_value.clone(),
        }),
    }
}
