use std::collections::BTreeMap;

use blox_language::ast;
use rust_decimal::Decimal;
use tracing::{trace, Level};

use crate::{
    expression::{
        casting::{cast_to_array, cast_to_number},
        condition::evaluate_condition,
        evaluate_expression,
        function_call::evaluate_function_call,
        method_call::evaluate_method_call,
    },
    program::evaluate_block,
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
        ast::ExpressionTerm::Array(array) => {
            let mut members = Vec::new();
            for member_expression in array.0.iter() {
                let value = evaluate_expression(member_expression, context)?;
                members.push(value);
            }
            Ok(Value::Array(members))
        }
        ast::ExpressionTerm::ArraySlice(ast::ArraySlice { base, start, end }) => {
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
        ast::ExpressionTerm::ArrayIndex(ast::ArrayIndex { base, index }) => {
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
        ast::ExpressionTerm::Object(ast::Object(members)) => {
            let mut object = BTreeMap::new();
            for (key, value_expression) in members.iter() {
                let value = evaluate_expression(value_expression, context)?;
                object.insert(key.clone(), value);
            }
            Ok(Value::Object(object))
        }
        ast::ExpressionTerm::ObjectIndex(ast::ObjectIndex { base, index }) => {
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
        ast::ExpressionTerm::If(ast::If {
            condition,
            body,
            elseif_branches,
            else_branch,
        }) => {
            if evaluate_condition(condition, context)? {
                return evaluate_block(body, context);
            }

            for elseif_branch in elseif_branches {
                if evaluate_condition(&elseif_branch.0, context)? {
                    return evaluate_block(&elseif_branch.1, context);
                }
            }

            if let Some(else_branch) = else_branch {
                return evaluate_block(else_branch, context);
            }

            Ok(Value::Void)
        }
        ast::ExpressionTerm::Lambda(definition) => Ok(Value::Function(Function {
            definition: definition.clone(),
            closure: context.scope.clone(),
        })),
    }?;

    trace!("{term} => {result}");

    Ok(result)
}
