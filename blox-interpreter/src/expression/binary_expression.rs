use blox_language::ast;
use tracing::{trace, Level};

use crate::{
    expression::{assign_to_expression, evaluate_expression},
    EvaluationContext, RuntimeError, Value,
};

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn evaluate_binary_expression(
    lhs: &Box<ast::Expression>,
    operator: &ast::Operator,
    rhs: &Box<ast::Expression>,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let lhs_value = evaluate_expression(lhs, context)?;
    let rhs_value = evaluate_expression(rhs, context)?;

    let result = match (&lhs_value, operator, &rhs_value) {
        (Value::Number(lhs), ast::Operator::Add, Value::Number(rhs)) => {
            Ok(Value::Number(lhs + rhs))
        }
        (Value::Number(lhs), ast::Operator::Subtract, Value::Number(rhs)) => {
            Ok(Value::Number(lhs - rhs))
        }
        (Value::Number(lhs), ast::Operator::Multiply, Value::Number(rhs)) => {
            Ok(Value::Number(lhs * rhs))
        }
        (Value::String(lhs), ast::Operator::Concatenate, Value::String(rhs)) => {
            Ok(Value::String(format!("{lhs}{rhs}")))
        }
        (Value::Array(lhs), ast::Operator::Concatenate, Value::Array(rhs)) => {
            let mut result = lhs.clone();
            result.extend(rhs.clone());
            Ok(Value::Array(result))
        }

        (Value::Array(lhs), ast::Operator::Equal, Value::Array(rhs)) => {
            Ok(Value::Boolean(lhs == rhs))
        }
        (Value::Boolean(lhs), ast::Operator::Equal, Value::Boolean(rhs)) => {
            Ok(Value::Boolean(lhs == rhs))
        }
        (Value::Number(lhs), ast::Operator::Equal, Value::Number(rhs)) => {
            Ok(Value::Boolean(lhs == rhs))
        }
        (Value::String(lhs), ast::Operator::Equal, Value::String(rhs)) => {
            Ok(Value::Boolean(lhs == rhs))
        }
        (Value::Symbol(lhs), ast::Operator::Equal, Value::Symbol(rhs)) => {
            Ok(Value::Boolean(lhs == rhs))
        }
        (_, ast::Operator::Equal, _) => Ok(Value::Boolean(false)),

        (Value::Array(lhs), ast::Operator::NotEqual, Value::Array(rhs)) => {
            Ok(Value::Boolean(lhs != rhs))
        }
        (Value::Boolean(lhs), ast::Operator::NotEqual, Value::Boolean(rhs)) => {
            Ok(Value::Boolean(lhs != rhs))
        }
        (Value::Number(lhs), ast::Operator::NotEqual, Value::Number(rhs)) => {
            Ok(Value::Boolean(lhs != rhs))
        }
        (Value::String(lhs), ast::Operator::NotEqual, Value::String(rhs)) => {
            Ok(Value::Boolean(lhs != rhs))
        }
        (Value::Symbol(lhs), ast::Operator::NotEqual, Value::Symbol(rhs)) => {
            Ok(Value::Boolean(lhs != rhs))
        }
        (_, ast::Operator::NotEqual, _) => Ok(Value::Boolean(false)),

        (Value::Number(lhs), ast::Operator::GreaterOrEqual, Value::Number(rhs)) => {
            Ok(Value::Boolean(lhs >= rhs))
        }
        (_, ast::Operator::GreaterOrEqual, _) => Ok(Value::Boolean(false)),

        (Value::Number(lhs), ast::Operator::GreaterThan, Value::Number(rhs)) => {
            Ok(Value::Boolean(lhs > rhs))
        }
        (_, ast::Operator::GreaterThan, _) => Ok(Value::Boolean(false)),

        (Value::Number(lhs), ast::Operator::LessOrEqual, Value::Number(rhs)) => {
            Ok(Value::Boolean(lhs <= rhs))
        }
        (_, ast::Operator::LessOrEqual, _) => Ok(Value::Boolean(false)),

        (Value::Number(lhs), ast::Operator::LessThan, Value::Number(rhs)) => {
            Ok(Value::Boolean(lhs < rhs))
        }
        (_, ast::Operator::LessThan, _) => Ok(Value::Boolean(false)),

        (Value::Array(lhs), ast::Operator::Append, rhs) => {
            let mut lhs = lhs.clone();
            lhs.push(rhs.clone());
            Ok(Value::Array(lhs))
        }

        (lhs_value, operator, rhs_value) => Err(RuntimeError::InvalidOperands {
            lhs_expression: *lhs.clone(),
            lhs_value: lhs_value.clone(),
            operator: operator.clone(),
            rhs_expression: *rhs.clone(),
            rhs_value: rhs_value.clone(),
        }),
    }?;

    // if operator is a mutating operator, update the lhs value in the scope
    match operator {
        ast::Operator::Append => {
            assign_to_expression(lhs, result.clone(), context)?;
        }

        _ => {}
    }

    trace!("{lhs_value} {operator} {rhs_value} => {result}");

    Ok(result)
}
