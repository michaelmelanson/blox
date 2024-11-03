use blox_language::ast;

use crate::{RuntimeError, Scope, Value};

pub fn evaluate_expression(
    expression: &ast::Expression,
    scope: &Scope,
) -> Result<Value, RuntimeError> {
    match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, scope),
        ast::Expression::Operator { lhs, operator, rhs } => {
            let lhs_value = evaluate_expression_term(lhs, scope)?;
            let rhs_value = evaluate_expression_term(rhs, scope)?;

            match operator {
                ast::Operator::Add => match (&lhs_value, &rhs_value) {
                    (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
                    _ => Err(RuntimeError::InvalidOperands {
                        lhs_expression: lhs.clone(),
                        lhs_value,
                        operator: operator.clone(),
                        rhs_expression: rhs.clone(),
                        rhs_value,
                    }),
                },
                ast::Operator::Multiply => match (&lhs_value, &rhs_value) {
                    (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),
                    _ => Err(RuntimeError::InvalidOperands {
                        lhs_expression: lhs.clone(),
                        lhs_value,
                        operator: operator.clone(),
                        rhs_expression: rhs.clone(),
                        rhs_value,
                    }),
                },
            }
        }
    }
}

pub fn evaluate_expression_term(
    term: &ast::ExpressionTerm,
    scope: &Scope,
) -> Result<Value, RuntimeError> {
    match term {
        ast::ExpressionTerm::Identifier(identifier) => scope.get_binding(&identifier.0).cloned(),
        ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Ok(Value::Number(*number)),
        ast::ExpressionTerm::Literal(ast::Literal::String(string)) => {
            Ok(Value::String(string.clone()))
        }
        ast::ExpressionTerm::Literal(ast::Literal::Symbol(string)) => {
            Ok(Value::Symbol(string.clone()))
        }
        ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, scope),
        ast::ExpressionTerm::FunctionCall(function_call) => todo!("function call"),
    }
}

#[cfg(test)]
mod tests {
    use blox_language::{
        ast::{self, Expression, Identifier},
        parse,
        parser::{self, Rule},
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

        let mut scope = Scope::default();
        scope.insert_binding(&Identifier("x".to_string()), Value::Number(55));

        let result = evaluate_expression(&expression, &scope);
        assert_eq!(result, Ok(Value::Number(56)));
    }

    #[test]
    fn test_evaluate_addition_identifier_identifier() {
        let expression = parse_expression("x + y".to_string()).expect("parse error");

        let mut scope = Scope::default();
        scope.insert_binding(&Identifier("x".to_string()), Value::Number(55));
        scope.insert_binding(&Identifier("y".to_string()), Value::Number(42));

        let result = evaluate_expression(&expression, &scope);
        assert_eq!(result, Ok(Value::Number(97)));
    }
}
