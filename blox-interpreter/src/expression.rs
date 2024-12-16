use std::collections::{BTreeMap, HashMap};

use blox_language::ast::{self, Argument, ArrayIndex, If, Object, ObjectIndex};
use rust_decimal::Decimal;
use tracing::trace;

use crate::{
    module::EvaluationContext, program::evaluate_block, value::Function, Intrinsic, RuntimeError,
    Value,
};

#[tracing::instrument(level = "trace", skip(context), ret)]
pub fn evaluate_expression(
    expression: &ast::Expression,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, context),
        ast::Expression::BinaryExpression(lhs, operator, rhs) => {
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
    }
}

pub fn assign_to_expression(
    target: &ast::Expression,
    value: Value,
    context: &mut EvaluationContext,
) -> Result<(), RuntimeError> {
    match target {
        ast::Expression::Term(ast::ExpressionTerm::Identifier(identifier)) => {
            context.scope.insert_binding(&identifier, value);
            Ok(())
        }
        ast::Expression::Term(ast::ExpressionTerm::ArrayIndex(ArrayIndex { base, index })) => {
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
        ast::Expression::Term(ast::ExpressionTerm::ObjectIndex(ObjectIndex { base, index })) => {
            let base_value = evaluate_expression(&base, context)?;

            match base_value {
                Value::Object(mut members) => {
                    members.insert(index.0.clone(), value);
                    assign_to_expression(&base, Value::Object(members), context)
                }
                base_value => Err(RuntimeError::NotAnObject {
                    object_expression: *base.clone(),
                    object_value: base_value.clone(),
                    key: index.0.clone(),
                }),
            }
        }

        target => Err(RuntimeError::LhsNotAssignable {
            expression: target.clone(),
            value: value.clone(),
        }),
    }
}

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
        ast::ExpressionTerm::ArrayIndex(ArrayIndex { base, index }) => {
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
        ast::ExpressionTerm::Object(Object(members)) => {
            let mut object = BTreeMap::new();
            for (key, value_expression) in members.iter() {
                let value = evaluate_expression(value_expression, context)?;
                object.insert(key.clone(), value);
            }
            Ok(Value::Object(object))
        }
        ast::ExpressionTerm::ObjectIndex(ObjectIndex { base, index }) => {
            let object_value = evaluate_expression(base, context)?;

            match object_value {
                Value::Object(ref members) => {
                    if let Some(value) = members.get(&index.0) {
                        Ok(value.clone())
                    } else {
                        Err(RuntimeError::ObjectKeyNotFound {
                            object_expression: *base.clone(),
                            object_value: object_value.clone(),
                            key: index.0.clone(),
                        })
                    }
                }
                object_value => Err(RuntimeError::NotAnObject {
                    object_expression: *base.clone(),
                    object_value: object_value.clone(),
                    key: index.0.clone(),
                }),
            }
        }
        ast::ExpressionTerm::If(If {
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

fn evaluate_condition(
    expression: &ast::Expression,
    context: &mut EvaluationContext,
) -> Result<bool, RuntimeError> {
    let condition_value = evaluate_expression(expression, context)?;

    let is_truthy = match condition_value {
        Value::Boolean(value) => value,
        Value::Number(number) => number.is_sign_positive() && !number.is_zero(),
        condition_value => {
            return Err(RuntimeError::InvalidCondition {
                condition_expression: expression.clone(),
                condition_value,
            });
        }
    };

    Ok(is_truthy)
}

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

            for Argument(name, rhs) in function_call.1.iter() {
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

#[cfg(test)]
mod tests {
    use blox_language::{
        ast::{Expression, Identifier},
        ParseError,
    };

    use crate::{EvaluationContext, Value};

    use super::evaluate_expression;

    fn parse_expression<'a>(code: String) -> Result<Expression, ParseError> {
        let parser = blox_language::Parser::new(&code);
        Ok(parser.parse_as_expression()?)
    }

    #[test]
    fn test_evaluate_addition_identifier_literal() {
        let expression = parse_expression("x + 1".to_string()).expect("parse error");

        let mut context = EvaluationContext::default();
        context
            .scope
            .insert_binding(&Identifier("x".to_string()), Value::Number(55.into()));

        let result = evaluate_expression(&expression, &mut context);
        assert_eq!(result, Ok(Value::Number(56.into())));
    }

    #[test]
    fn test_evaluate_addition_identifier_identifier() {
        let expression = parse_expression("x + y".to_string()).expect("parse error");

        let mut context = EvaluationContext::default();
        context
            .scope
            .insert_binding(&Identifier("x".to_string()), Value::Number(55.into()));
        context
            .scope
            .insert_binding(&Identifier("y".to_string()), Value::Number(42.into()));

        let result = evaluate_expression(&expression, &mut context);
        assert_eq!(result, Ok(Value::Number(97.into())));
    }
}
