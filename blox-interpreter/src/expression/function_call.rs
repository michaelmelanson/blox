use std::collections::HashMap;

use blox_language::ast;
use tracing::{trace, Level};

use crate::{
    expression::evaluate_expression, program::evaluate_block, value::Function, EvaluationContext,
    Intrinsic, RuntimeError, Value,
};

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn evaluate_function_call(
    function_call: &ast::FunctionCall,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let function = evaluate_expression(&function_call.0, context)?;

    let result = match function {
        Value::Function(Function {
            definition,
            closure,
        }) => {
            let mut call_context = context.child_with_scope(closure.child());

            for (parameter, argument) in definition.parameters.iter().zip(&function_call.1) {
                let name = &parameter.0;
                let value = evaluate_expression(&argument.1, context)?;

                call_context.scope.insert_binding(name, value);
            }

            evaluate_block(&definition.body, &mut call_context)
        }
        Value::Intrinsic(Intrinsic {
            id: _,
            name: _,
            function,
        }) => {
            let mut parameters = HashMap::new();

            for ast::Argument(name, rhs) in function_call.1.iter() {
                let value = evaluate_expression(&rhs, context)?;
                parameters.insert(name.clone(), value);
            }

            function(parameters)
        }
        _ => Err(RuntimeError::NotAFunction {
            callee: *function_call.0.clone(),
            value: function.clone(),
        }),
    }?;

    trace!("{function_call} returned {result}");

    Ok(result)
}
