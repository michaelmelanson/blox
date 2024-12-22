use std::collections::BTreeMap;

use blox_language::ast;

use crate::{expression::evaluate_expression, EvaluationContext, RuntimeError, Value};

pub fn evaluate_object(
    object: &ast::Object,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let ast::Object(members) = object;
    let mut object = BTreeMap::new();
    for (key, value_expression) in members.iter() {
        let value = evaluate_expression(value_expression, context)?;
        object.insert(key.clone(), value);
    }
    Ok(Value::Object(object))
}
