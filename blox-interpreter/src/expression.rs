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
    use blox_language::ast;

    use crate::{Scope, Value};

    use super::evaluate_expression;

    #[test]
    fn test_evaluate_addition() {
        let expression = ast::Expression::Operator {
            lhs: ast::ExpressionTerm::Identifier(ast::Identifier("x".to_string())),
            operator: ast::Operator::Add,
            rhs: ast::ExpressionTerm::Literal(ast::Literal::Number(1))
        };

        let mut scope = Scope::default();
        scope.insert_binding("x".to_string(), Value::Number(55));

        let result = evaluate_expression(&expression, &scope);

        assert_eq!(result, Some(Value::Number(56)));
    }
}
