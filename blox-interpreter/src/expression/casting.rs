use blox_language::ast;
use rust_decimal::Decimal;

use crate::{RuntimeError, Value};

pub fn cast_to_array(value: Value, context: &ast::Expression) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(array) => Ok(array),
        value => Err(RuntimeError::NotAnArray {
            expression: context.clone(),
            value,
        }),
    }
}

pub fn cast_to_number(value: Value, context: &ast::Expression) -> Result<Decimal, RuntimeError> {
    match value {
        Value::Number(number) => Ok(number),
        value => Err(RuntimeError::NotANumber {
            expression: context.clone(),
            value,
        }),
    }
}
