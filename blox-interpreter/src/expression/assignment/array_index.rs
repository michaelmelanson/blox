use blox_language::ast;

use crate::{expression::evaluate_expression, EvaluationContext, RuntimeError, Value};

use super::assign_to_expression;

pub fn assign_to_array_index(
    array_index: &ast::ArrayIndex,
    value: Value,
    context: &mut EvaluationContext,
) -> Result<(), RuntimeError> {
    let ast::ArrayIndex { base, index } = array_index;

    let base_value = evaluate_expression(&base, context)?;
    let index_value = evaluate_expression(&index, context)?;

    match (&base_value, &index_value) {
        (Value::Array(ref members), Value::Number(idx)) => {
            let Ok(idx): rust_decimal::Result<usize> = (*idx).try_into() else {
                return Err(RuntimeError::InvalidArrayIndex {
                    array_expression: *base.clone(),
                    array_value: base_value.clone(),
                    index_expression: *index.clone(),
                    index_value: index_value.clone(),
                });
            };

            if idx < members.len() {
                let mut members = members.clone();
                members[idx] = value;
                assign_to_expression(&base, Value::Array(members), context)
            } else {
                Err(RuntimeError::ArrayIndexOutOfBounds {
                    array_expression: *base.clone(),
                    array_value: base_value.clone(),
                    index_expression: *index.clone(),
                    index_value: index_value.clone(),
                })
            }
        }
        (base_value, index_value) => Err(RuntimeError::InvalidArrayIndex {
            array_expression: *base.clone(),
            array_value: base_value.clone(),
            index_expression: *index.clone(),
            index_value: index_value.clone(),
        }),
    }
}
