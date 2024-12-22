use blox_language::ast;
use tracing::Level;

use crate::{EvaluationContext, RuntimeError, Value};

use super::evaluate_expression;

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
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
        ast::Expression::Term(ast::ExpressionTerm::ArrayIndex(ast::ArrayIndex { base, index })) => {
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
        ast::Expression::Term(ast::ExpressionTerm::ObjectIndex(ast::ObjectIndex {
            base,
            index,
        })) => {
            let base_value = evaluate_expression(&base, context)?;

            match base_value {
                Value::Object(mut members) => {
                    members.insert(index.name.clone(), value);
                    assign_to_expression(&base, Value::Object(members), context)
                }
                base_value => Err(RuntimeError::NotAnObject {
                    object_expression: *base.clone(),
                    object_value: base_value.clone(),
                    key: index.name.clone(),
                }),
            }
        }

        target => Err(RuntimeError::LhsNotAssignable {
            expression: target.clone(),
            value: value.clone(),
        }),
    }
}
