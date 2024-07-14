pub mod ast;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub program);

pub type ParseError<'a> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

pub fn parse<'a>(code: &'a str) -> std::result::Result<ast::Program, ParseError<'a>> {
    program::ProgramParser::new().parse(code)
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::parse;

    #[test]
    fn parse_let_bindings() {
        let actual = parse(&"let test = 55".to_string()).expect("parse error");
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
        let actual = parse(&"let test = 55 + 42".to_string()).expect("parse error");
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
        let actual = parse(&"let test = (1 * 2) + 3".to_string()).expect("parse error");
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
        let actual = parse(&"let test = :symbol".to_string()).expect("parse error");
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
