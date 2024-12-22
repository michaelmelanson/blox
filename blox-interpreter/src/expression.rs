mod assignment;
mod binary_expression;
mod casting;
mod condition;
mod expression_term;
mod function_call;
mod method_call;

use blox_language::ast;
use tracing::Level;

use crate::{module::EvaluationContext, RuntimeError, Value};

use self::{
    assignment::assign_to_expression, binary_expression::evaluate_binary_expression,
    expression_term::evaluate_expression_term,
};

#[tracing::instrument(skip(context), ret(level=Level::TRACE), err(level=Level::DEBUG))]
pub fn evaluate_expression(
    expression: &ast::Expression,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    // grow the stack here if needed, to avoid stack overflows on deeply nested expressions
    const STACK_RED_ZONE: usize = 128 * 1024; // grow when there's less than this amount remaining
    const STACK_BLOCK_SIZE: usize = 1024 * 1024; // grow by 1MB at a time

    stacker::maybe_grow(STACK_RED_ZONE, STACK_BLOCK_SIZE, || match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, context),
        ast::Expression::BinaryExpression(lhs, operator, rhs) => {
            evaluate_binary_expression(lhs, operator, rhs, context)
        }
    })
}

#[cfg(test)]
mod tests {
    use blox_language::{
        ast::{Expression, Identifier},
        error::ParseError,
        parser::Parser,
    };

    use crate::{EvaluationContext, Value};

    use super::evaluate_expression;

    fn parse_expression<'a>(code: String) -> Result<Expression, ParseError> {
        let parser = Parser::new("<test>", &code);
        Ok(parser.parse_as_expression()?)
    }

    #[test]
    fn test_evaluate_addition_identifier_literal() {
        let expression = parse_expression("x + 1".to_string()).expect("parse error");

        let mut context = EvaluationContext::default();
        context.scope.insert_binding(
            &Identifier {
                name: "x".to_string(),
            },
            Value::Number(55.into()),
        );

        let result = evaluate_expression(&expression, &mut context);
        assert_eq!(result, Ok(Value::Number(56.into())));
    }

    #[test]
    fn test_evaluate_addition_identifier_identifier() {
        let expression = parse_expression("x + y".to_string()).expect("parse error");

        let mut context = EvaluationContext::default();
        context.scope.insert_binding(
            &Identifier {
                name: "x".to_string(),
            },
            Value::Number(55.into()),
        );
        context.scope.insert_binding(
            &Identifier {
                name: "y".to_string(),
            },
            Value::Number(42.into()),
        );

        let result = evaluate_expression(&expression, &mut context);
        assert_eq!(result, Ok(Value::Number(97.into())));
    }
}
