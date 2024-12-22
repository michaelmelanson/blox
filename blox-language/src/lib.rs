pub mod ast;
pub mod error;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::{ast, error::ParseError, parser::Parser};

    fn parse(input: &str) -> Result<ast::Program, ParseError> {
        let parser = Parser::new(input);
        parser.parse()
    }

    #[test]
    fn parse_let_bindings() {
        let actual = parse(&"let test = 55".to_string()).expect("parse error");
        assert_eq!(
            ast::Program(ast::Block(vec![ast::Statement::Binding(
                ast::Identifier("test".to_string()),
                ast::Expression::Term(ast::ExpressionTerm::Literal(ast::Literal::Number(
                    55.into()
                )))
            )])),
            actual
        );
    }

    #[test]
    fn parse_expressions() {
        let actual = parse(&"let test = 55 + 42".to_string()).expect("parse error");
        assert_eq!(
            ast::Program(ast::Block(vec![ast::Statement::Binding(
                ast::Identifier("test".to_string()),
                ast::Expression::BinaryExpression(
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                        ast::Literal::Number(55.into())
                    ))),
                    ast::Operator::Add,
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                        ast::Literal::Number(42.into())
                    )))
                )
            )])),
            actual
        );
    }

    #[test]
    fn test_nested_expressions() {
        let actual = parse(&"let test = (1 * 2) + 3".to_string()).expect("parse error");
        assert_eq!(
            ast::Program(ast::Block(vec![ast::Statement::Binding(
                ast::Identifier("test".to_string()),
                ast::Expression::BinaryExpression(
                    Box::new(ast::Expression::BinaryExpression(
                        Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                            ast::Literal::Number(1.into())
                        ))),
                        ast::Operator::Multiply,
                        Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                            ast::Literal::Number(2.into())
                        )))
                    )),
                    ast::Operator::Add,
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                        ast::Literal::Number(3.into())
                    )))
                )
            )])),
            actual
        );
    }

    #[test]
    fn test_symbols() {
        let actual = parse(&"let test = :symbol".to_string()).expect("parse error");
        assert_eq!(
            ast::Program(ast::Block(vec![ast::Statement::Binding(
                ast::Identifier("test".to_string()),
                ast::Expression::Term(ast::ExpressionTerm::Literal(ast::Literal::Symbol(
                    "symbol".to_string()
                )))
            )])),
            actual
        );
    }
}
