use std::{collections::BTreeMap, sync::Arc};

use blox_language::ast::{self, ArrayIndex, If, Object, ObjectIndex};

use crate::{program::evaluate_block, value::Function, RuntimeError, Scope, Value};

pub fn evaluate_expression(
    expression: &ast::Expression,
    scope: &mut Arc<Scope>,
) -> Result<Value, RuntimeError> {
    match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, scope),
        ast::Expression::Operator(lhs, operator, rhs) => {
            let lhs_value = evaluate_expression(lhs, scope)?;
            let rhs_value = evaluate_expression(rhs, scope)?;

            match (lhs_value, operator, rhs_value) {
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

                (lhs_value, operator, rhs_value) => Err(RuntimeError::InvalidOperands {
                    lhs_expression: *lhs.clone(),
                    lhs_value,
                    operator: operator.clone(),
                    rhs_expression: *rhs.clone(),
                    rhs_value,
                }),
            }
        }
    }
}

pub fn evaluate_expression_term(
    term: &ast::ExpressionTerm,
    scope: &mut Arc<Scope>,
) -> Result<Value, RuntimeError> {
    match term {
        ast::ExpressionTerm::Identifier(identifier) => scope.get_binding(&identifier),
        ast::ExpressionTerm::Literal(ast::Literal::Boolean(value)) => Ok(Value::Boolean(*value)),
        ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Ok(Value::Number(*number)),
        ast::ExpressionTerm::Literal(ast::Literal::String(string)) => {
            Ok(Value::String(string.clone()))
        }
        ast::ExpressionTerm::Literal(ast::Literal::Symbol(string)) => {
            Ok(Value::Symbol(string.clone()))
        }
        ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, scope),
        ast::ExpressionTerm::FunctionCall(function_call) => {
            evaluate_function_call(function_call, scope)
        }
        ast::ExpressionTerm::Array(array) => {
            let mut members = Vec::new();
            for member_expression in array.0.iter() {
                let value = evaluate_expression(member_expression, scope)?;
                members.push(value);
            }
            Ok(Value::Array(members))
        }
        ast::ExpressionTerm::ArrayIndex(ArrayIndex { array, index }) => {
            let array_value = evaluate_expression_term(array, scope)?;
            let index_value = evaluate_expression(index, scope)?;

            match (&array_value, &index_value) {
                (Value::Array(ref members), Value::Number(idx)) => {
                    let Ok(idx): rust_decimal::Result<usize> = (*idx).try_into() else {
                        return Err(RuntimeError::InvalidArrayIndex {
                            array_expression: *array.clone(),
                            array_value: array_value.clone(),
                            index_expression: *index.clone(),
                            index_value: index_value.clone(),
                        });
                    };

                    if idx < members.len() {
                        Ok(members[idx].clone())
                    } else {
                        Err(RuntimeError::ArrayIndexOutOfBounds {
                            array_expression: *array.clone(),
                            array_value: array_value.clone(),
                            index_expression: *index.clone(),
                            index_value: index_value.clone(),
                        })
                    }
                }
                (array_value, index_value) => Err(RuntimeError::InvalidArrayIndex {
                    array_expression: *array.clone(),
                    array_value: array_value.clone(),
                    index_expression: *index.clone(),
                    index_value: index_value.clone(),
                }),
            }
        }
        ast::ExpressionTerm::Object(Object(members)) => {
            let mut object = BTreeMap::new();
            for (key, value_expression) in members.iter() {
                let value = evaluate_expression(value_expression, scope)?;
                object.insert(key.clone(), value);
            }
            Ok(Value::Object(object))
        }
        ast::ExpressionTerm::ObjectIndex(ObjectIndex { object, key }) => {
            let object_value = evaluate_expression_term(object, scope)?;

            match object_value {
                Value::Object(ref members) => {
                    if let Some(value) = members.get(key) {
                        Ok(value.clone())
                    } else {
                        Err(RuntimeError::ObjectKeyNotFound {
                            object_expression: *object.clone(),
                            object_value: object_value.clone(),
                            key: key.clone(),
                        })
                    }
                }
                object_value => Err(RuntimeError::NotAnObject {
                    object_expression: *object.clone(),
                    object_value: object_value.clone(),
                    key: key.clone(),
                }),
            }
        }
        ast::ExpressionTerm::If(If {
            condition,
            then_branch,
            elseif_branches,
            else_branch,
        }) => {
            if evaluate_condition(condition, scope)? {
                return evaluate_block(then_branch, scope);
            }

            for elseif_branch in elseif_branches {
                if evaluate_condition(&elseif_branch.0, scope)? {
                    return evaluate_block(&elseif_branch.1, scope);
                }
            }

            if let Some(else_branch) = else_branch {
                return evaluate_block(else_branch, scope);
            }

            Ok(Value::Void)
        }
    }
}

fn evaluate_condition(
    expression: &ast::Expression,
    scope: &mut Arc<Scope>,
) -> Result<bool, RuntimeError> {
    let condition_value = evaluate_expression(expression, scope)?;

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

pub fn evaluate_function_call(
    function_call: &ast::FunctionCall,
    scope: &mut Arc<Scope>,
) -> Result<Value, RuntimeError> {
    let function = scope.get_binding(&function_call.0)?.clone();

    match function {
        Value::Function(Function {
            definition,
            closure,
        }) => {
            let mut call_scope = closure.child();

            for (parameter, argument) in definition.parameters.iter().zip(&function_call.1) {
                let value = evaluate_expression(&argument.1, scope)?;
                call_scope.insert_binding(&parameter.0, value);
            }

            evaluate_block(&definition.body, &mut call_scope)
        }
        _ => Err(RuntimeError::NotAFunction {
            identifier: function_call.0.clone(),
            value: function.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use blox_language::{
        ast::{Expression, Identifier},
        ParseError,
    };

    use crate::{Scope, Value};

    use super::evaluate_expression;

    fn parse_expression<'a>(code: String) -> Result<Expression, ParseError> {
        Ok(blox_language::parse_expression_string(&code)?)
    }

    #[test]
    fn test_evaluate_addition_identifier_literal() {
        let expression = parse_expression("x + 1".to_string()).expect("parse error");

        let mut scope = Arc::new(Scope::default());
        scope.insert_binding(&Identifier("x".to_string()), Value::Number(55.into()));

        let result = evaluate_expression(&expression, &mut scope);
        assert_eq!(result, Ok(Value::Number(56.into())));
    }

    #[test]
    fn test_evaluate_addition_identifier_identifier() {
        let expression = parse_expression("x + y".to_string()).expect("parse error");

        let mut scope = Arc::new(Scope::default());
        scope.insert_binding(&Identifier("x".to_string()), Value::Number(55.into()));
        scope.insert_binding(&Identifier("y".to_string()), Value::Number(42.into()));

        let result = evaluate_expression(&expression, &mut scope);
        assert_eq!(result, Ok(Value::Number(97.into())));
    }
}
