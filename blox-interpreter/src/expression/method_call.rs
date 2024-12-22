use blox_language::ast;
use tracing::Level;

use crate::{EvaluationContext, RuntimeError, Value};

use super::function_call::evaluate_function_call;

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn evaluate_method_call(
    method_call: &ast::MethodCall,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let function = context.scope.get_binding(&method_call.function)?;

    let Value::Function(function) = function else {
        return Err(RuntimeError::NotAFunction {
            callee: ast::Expression::Term(ast::ExpressionTerm::Identifier(
                method_call.function.clone(),
            )),
            value: function,
        });
    };
    let Some(self_param) = function.definition.parameters.first() else {
        return Err(RuntimeError::MethodCallWithoutSelf {
            method: method_call.function.clone(),
        });
    };

    let mut arguments = vec![ast::Argument(
        self_param.0.clone(),
        *method_call.base.clone(),
    )];
    arguments.append(&mut method_call.arguments.clone());

    let function_call = ast::FunctionCall(
        Box::new(ast::Expression::Term(ast::ExpressionTerm::Identifier(
            method_call.function.clone(),
        ))),
        arguments,
    );

    evaluate_function_call(&function_call, context)
}
