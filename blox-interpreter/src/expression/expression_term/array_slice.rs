use blox_language::ast;
use rust_decimal::Decimal;

use crate::{
    expression::{
        casting::{cast_to_array, cast_to_number},
        evaluate_expression,
    },
    EvaluationContext, RuntimeError, Value,
};

pub fn evaluate_array_slice(
    array_slice: &ast::ArraySlice,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let ast::ArraySlice { base, start, end } = array_slice;
    let base_value: Vec<Value> = cast_to_array(evaluate_expression(base, context)?, base)?;
    let start_value: Decimal = if let Some(start) = start {
        cast_to_number(evaluate_expression(start, context)?, start)?
    } else {
        0.into()
    };

    let end_value: Option<Decimal> = if let Some(end) = end {
        Some(cast_to_number(evaluate_expression(end, context)?, end)?)
    } else {
        None
    };

    let iter = base_value.iter().skip(start_value.try_into().unwrap());

    let result: Vec<Value> = if let Some(end_value) = end_value {
        let start_index: usize = start_value.try_into().unwrap();
        let end_index: usize = end_value.try_into().unwrap();

        iter.take(end_index - start_index)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        iter.cloned().collect::<Vec<_>>()
    };

    Ok(Value::Array(result))
}
