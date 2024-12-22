pub mod ast;
pub mod error;
pub mod location;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::{ast, error::ParseError, location::Location, parser::Parser};

    fn parse(input: &str) -> Result<ast::Program, ParseError> {
        let parser = Parser::new("<test>", input);
        parser.parse()
    }

    #[test]
    fn parse_records_locations() {
        let actual = parse(&"let test = 55".to_string()).expect("parse error");
        assert_eq!(
            Location {
                file: "<test>".to_string(),
                range: tree_sitter::Range {
                    start_byte: 0,
                    end_byte: 13,
                    start_point: tree_sitter::Point { row: 0, column: 0 },
                    end_point: tree_sitter::Point { row: 0, column: 13 }
                }
            },
            actual.location
        );
    }

    #[test]
    fn parse_let_bindings() {
        let actual = parse(&"let test = 55".to_string()).expect("parse error");
        assert_eq!(
            ast::Block(vec![ast::Statement::Binding(
                ast::Identifier {
                    name: "test".to_string()
                },
                ast::Expression::Term(ast::ExpressionTerm::Literal(ast::Literal::Number(
                    55.into()
                )))
            )]),
            actual.block
        );
    }

    #[test]
    fn parse_expressions() {
        let actual = parse(&"let test = 55 + 42".to_string()).expect("parse error");
        assert_eq!(
            ast::Block(vec![ast::Statement::Binding(
                ast::Identifier {
                    name: "test".to_string()
                },
                ast::Expression::BinaryExpression(
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                        ast::Literal::Number(55.into())
                    ))),
                    ast::Operator::Add,
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                        ast::Literal::Number(42.into())
                    )))
                )
            )]),
            actual.block
        );
    }

    #[test]
    fn test_nested_expressions() {
        let actual = parse(&"let test = (1 * 2) + 3".to_string()).expect("parse error");
        assert_eq!(
            ast::Block(vec![ast::Statement::Binding(
                ast::Identifier {
                    name: "test".to_string()
                },
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
            )]),
            actual.block
        );
    }

    #[test]
    fn test_symbols() {
        let actual = parse(&"let test = :symbol".to_string()).expect("parse error");
        assert_eq!(
            ast::Block(vec![ast::Statement::Binding(
                ast::Identifier {
                    name: "test".to_string()
                },
                ast::Expression::Term(ast::ExpressionTerm::Literal(ast::Literal::Symbol(
                    "symbol".to_string()
                )))
            )]),
            actual.block
        );
    }
}
