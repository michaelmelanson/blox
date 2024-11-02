use blox_language::ast;

use crate::{Scope, Value};

pub fn evaluate_expression(expression: &ast::Expression, scope: &Scope) -> Option<Value> {
    match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, scope),
        ast::Expression::Operator { lhs, operator, rhs } => {
            let lhs_value = evaluate_expression_term(lhs, scope);
            let rhs_value = evaluate_expression_term(rhs, scope);

            match operator {
                ast::Operator::Add => match (lhs_value, rhs_value) {
                    (Some(Value::Number(lhs)), Some(Value::Number(rhs))) => {
                        Some(Value::Number(lhs + &rhs))
                    }
                    _ => None,
                },
                ast::Operator::Multiply => match (lhs_value, rhs_value) {
                    (Some(Value::Number(lhs)), Some(Value::Number(rhs))) => {
                        Some(Value::Number(lhs * &rhs))
                    }
                    _ => None,
                },
            }
        }
    }
}

pub fn evaluate_expression_term(term: &ast::ExpressionTerm, scope: &Scope) -> Option<Value> {
    match term {
        ast::ExpressionTerm::Identifier(identifier) => scope.bindings.get(identifier).cloned(),
        ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Some(Value::Number(*number)),
        ast::ExpressionTerm::Literal(ast::Literal::String(string)) => {
            Some(Value::String(string.clone()))
        }
        ast::ExpressionTerm::Literal(ast::Literal::Symbol(string)) => {
            Some(Value::Symbol(string.clone()))
        }
        ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, scope),
    }
}

#[cfg(test)]
mod tests {
    use blox_language::{
        ast::{self, Expression},
        parse,
    };

    use crate::{Scope, Value};

    use super::evaluate_expression;

    fn parse_expression<'a>(expression_code: String) -> Result<Expression, String> {
        let code = format!("let __TEST = {}", expression_code);
        match parse(&code) {
            Ok(program) => {
                let statement = program.block.statements.first().expect("no statements");

                match statement {
                    ast::Statement::Binding { lhs: _, rhs } => Ok(rhs.clone()),
                    statement => panic!(
                        "expression produced wrong kind of statement: {:?}",
                        statement
                    ),
                }
            }

            Err(error) => Err(error.to_string()),
        }
    }

    #[test]
    fn test_evaluate_addition_identifier_literal() {
        let expression = parse_expression("x + 1".to_string()).expect("parse error");

        let mut scope = Scope::default();
        scope.insert_binding("x".to_string(), Value::Number(55));

        let result = evaluate_expression(&expression, &scope);
        assert_eq!(result, Some(Value::Number(56)));
    }

    #[test]
    fn test_evaluate_addition_identifier_identifier() {
        let expression = parse_expression("x + y".to_string()).expect("parse error");

        let mut scope = Scope::default();
        scope.insert_binding("x".to_string(), Value::Number(55));
        scope.insert_binding("y".to_string(), Value::Number(42));

        let result = evaluate_expression(&expression, &scope);
        assert_eq!(result, Some(Value::Number(97)));
    }
}
