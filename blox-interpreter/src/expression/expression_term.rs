mod array;
mod array_index;
mod array_slice;
mod if_term;
mod object;
mod object_index;

use array::evaluate_array;
use array_index::evaluate_array_index;
use array_slice::evaluate_array_slice;
use blox_language::ast;
use if_term::evaluate_if_term;
use object::evaluate_object;
use object_index::evaluate_object_index;
use tracing::{trace, Level};

use crate::{
    expression::{
        evaluate_expression, function_call::evaluate_function_call,
        method_call::evaluate_method_call,
    },
    value::Function,
    EvaluationContext, RuntimeError, Value,
};

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn evaluate_expression_term(
    term: &ast::ExpressionTerm,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let result = match term {
        ast::ExpressionTerm::Identifier(identifier) => context.scope.get_binding(&identifier),
        ast::ExpressionTerm::Literal(ast::Literal::Boolean(value)) => Ok(Value::Boolean(*value)),
        ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Ok(Value::Number(*number)),
        ast::ExpressionTerm::Literal(ast::Literal::String(string)) => {
            Ok(Value::String(string.clone()))
        }
        ast::ExpressionTerm::Literal(ast::Literal::Symbol(string)) => {
            Ok(Value::Symbol(string.clone()))
        }
        ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, context),
        ast::ExpressionTerm::MethodCall(method_call) => evaluate_method_call(method_call, context),
        ast::ExpressionTerm::FunctionCall(function_call) => {
            evaluate_function_call(function_call, context)
        }
        ast::ExpressionTerm::Array(array) => evaluate_array(array, context),
        ast::ExpressionTerm::ArraySlice(array_slice) => evaluate_array_slice(array_slice, context),
        ast::ExpressionTerm::ArrayIndex(array_index) => evaluate_array_index(array_index, context),
        ast::ExpressionTerm::Object(object) => evaluate_object(object, context),
        ast::ExpressionTerm::ObjectIndex(object_index) => {
            evaluate_object_index(object_index, context)
        }
        ast::ExpressionTerm::If(if_term) => evaluate_if_term(if_term, context),
        ast::ExpressionTerm::Lambda(definition) => Ok(Value::Function(Function {
            definition: definition.clone(),
            closure: context.scope.clone(),
        })),
    }?;

    trace!("{term} => {result}");

    Ok(result)
}
