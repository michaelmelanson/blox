pub mod ast;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub program);

pub type ParseError<'a> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

#[cfg(test)]
mod tests {
    use crate::program;
    use crate::{ast, ParseError};

    fn parse(code: &str) -> std::result::Result<ast::Program, ParseError<'_>> {
        program::ProgramParser::new().parse(code)
    }

    #[test]
    fn parse_let_bindings() {
        let actual = parse("let test = 55").expect("parse error");
        assert_eq!(
            ast::Program {
                block: ast::Block {
                    statements: vec![ast::Statement::Binding {
                        lhs: ast::Identifier("test".to_string()),
                        rhs: ast::Expression::Term(ast::ExpressionTerm::Literal(
                            ast::Literal::Number(55)
                        ))
                    }]
                }
            },
            actual
        );
    }

    #[test]
    fn parse_expressions() {
        let actual = parse("let test = 55 + 42").expect("parse error");
        assert_eq!(
            ast::Program {
                block: ast::Block {
                    statements: vec![ast::Statement::Binding {
                        lhs: ast::Identifier("test".to_string()),
                        rhs: ast::Expression::Operator {
                            lhs: ast::ExpressionTerm::Literal(ast::Literal::Number(55)),
                            operator: ast::Operator::Add,
                            rhs: ast::ExpressionTerm::Literal(ast::Literal::Number(42))
                        }
                    }]
                }
            },
            actual
        );
    }

    #[test]
    fn test_nested_expressions() {
        let actual = parse("let test = (1 * 2) + 3").expect("parse error");
        assert_eq!(
            ast::Program {
                block: ast::Block {
                    statements: vec![ast::Statement::Binding {
                        lhs: ast::Identifier("test".to_string()),
                        rhs: ast::Expression::Operator {
                            lhs: ast::ExpressionTerm::Expression(Box::new(
                                ast::Expression::Operator {
                                    lhs: ast::ExpressionTerm::Literal(ast::Literal::Number(1)),
                                    operator: ast::Operator::Multiply,
                                    rhs: ast::ExpressionTerm::Literal(ast::Literal::Number(2))
                                }
                            )),
                            operator: ast::Operator::Add,
                            rhs: ast::ExpressionTerm::Literal(ast::Literal::Number(3))
                        }
                    }]
                }
            },
            actual
        );
    }

    #[test]
    fn test_symbols() {
        let actual = parse("let test = :symbol").expect("parse error");
        assert_eq!(
            ast::Program {
                block: ast::Block {
                    statements: vec![ast::Statement::Binding {
                        lhs: ast::Identifier("test".to_string()),
                        rhs: ast::Expression::Term(ast::ExpressionTerm::Literal(
                            ast::Literal::Symbol("symbol".to_string())
                        ))
                    }]
                }
            },
            actual
        );
    }
}
