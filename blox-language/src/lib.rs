pub mod ast;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub program);

pub type ParseError<'a> = lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

#[cfg(test)]
mod tests {
    use crate::{ParseError, ast};
    use crate::program;

    fn parse(code: &str) -> std::result::Result<ast::Program, ParseError<'_>> {
        program::ProgramParser::new().parse(code)
    }
    
    #[test]
    fn parse_let_bindings() {
        let actual = parse("let test = 55").expect("parse error");
        assert_eq!(ast::Program {
            block: ast::Block {
                statements: vec![
                    ast::Statement::Binding {
                        lhs: ast::Identifier("test".to_string()),
                        rhs: ast::Expression::Term(
                            ast::ExpressionTerm::Literal(ast::Literal::Number(55))
                        )
                    }
                ]
            }
        }, actual);
    }
}
