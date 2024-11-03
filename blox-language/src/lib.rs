use pest::Parser;

pub mod ast;
pub mod parser;

#[derive(Debug)]
pub struct ParseError(pest::error::Error<parser::Rule>);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<pest::error::Error<parser::Rule>> for ParseError {
    fn from(err: pest::error::Error<parser::Rule>) -> Self {
        ParseError(err)
    }
}

pub fn parse(code: &str) -> std::result::Result<ast::Program, ParseError> {
    let mut result = parser::BloxParser::parse(parser::Rule::program, code)?;
    assert_eq!(result.len(), 1);

    let Some(pair) = result.next() else {
        panic!("expected a single program rule, found none");
    };

    Ok(parse_program(pair)?)
}

fn parse_program(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Program, ParseError> {
    let mut block = None;
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::block => {
                block = Some(parse_block(inner_pair)?);
            }
            parser::Rule::EOI => {}
            rule => unimplemented!("program rule: {rule:?}"),
        }
    }

    Ok(ast::Program {
        block: block.expect("expected block"),
    })
}

fn parse_block(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Block, ParseError> {
    let mut statements = vec![];
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::statement => {
                statements.push(parse_statement(inner_pair)?);
            }
            rule => unimplemented!("block rule: {rule:?}"),
        }
    }

    Ok(ast::Block { statements })
}

fn parse_statement(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Statement, ParseError> {
    let mut result = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::definition_statement => {
                let definition = parse_definition(inner_pair)?;
                result = Some(ast::Statement::Definition(definition));
            }
            parser::Rule::binding_statement => {
                let (lhs, rhs) = parse_binding(inner_pair)?;
                result = Some(ast::Statement::Binding { lhs, rhs });
            }
            parser::Rule::expression_statement => {
                let expression = parse_expression_statement(inner_pair)?;
                result = Some(ast::Statement::Expression(expression));
            }
            rule => unimplemented!("statement rule: {rule:?}"),
        }
    }

    Ok(result.expect("expected statement"))
}

fn parse_definition(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Definition, ParseError> {
    let mut name = None;
    let mut parameters = vec![];
    let mut body = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::identifier => {
                name = Some(parse_identifier(inner_pair)?);
            }
            parser::Rule::parameter => {
                parameters.push(parse_parameter(inner_pair)?);
            }
            parser::Rule::block => {
                body = Some(parse_block(inner_pair)?);
            }
            rule => unimplemented!("definition rule: {rule:?}"),
        }
    }

    Ok(ast::Definition {
        name: name.expect("expected name"),
        parameters,
        body: body.expect("expected body"),
    })
}

fn parse_parameter(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Parameter, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        parser::Rule::identifier => Ok(ast::Parameter(parse_identifier(inner_pair)?)),
        rule => unimplemented!("parameter rule: {rule:?}"),
    }
}

fn parse_binding(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<(ast::Identifier, ast::Expression), ParseError> {
    let mut lhs = None;
    let mut rhs = None;
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::identifier => {
                lhs = Some(parse_identifier(inner_pair)?);
            }
            parser::Rule::expression => {
                rhs = Some(parse_expression(inner_pair)?);
            }
            rule => unimplemented!("binding rule: {rule:?}"),
        }
    }

    Ok((lhs.expect("expected lhs"), rhs.expect("expected rhs")))
}

fn parse_expression_statement(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Expression, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        parser::Rule::expression => parse_expression(inner_pair),
        rule => unimplemented!("expression statement rule: {rule:?}"),
    }
}

pub fn parse_expression_string(code: &str) -> std::result::Result<ast::Expression, ParseError> {
    let mut result = parser::BloxParser::parse(parser::Rule::expression, code)?;
    println!("Parse result: {result}");
    assert_eq!(result.len(), 1);

    let Some(pair) = result.next() else {
        panic!("expected a single expression rule, found none");
    };

    parse_expression(pair)
}

fn parse_expression(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Expression, ParseError> {
    let mut lhs = None;
    let mut operator = None;
    let mut rhs = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::expression_term => {
                if lhs == None {
                    lhs = Some(parse_expression_term(inner_pair)?);
                } else if rhs == None {
                    rhs = Some(parse_expression_term(inner_pair)?);
                } else {
                    unimplemented!("unexpected expression term");
                }
            }
            parser::Rule::operator => {
                operator = Some(parse_operator(inner_pair)?);
            }
            rule => unimplemented!("expression rule: {rule:?}"),
        }
    }

    if let Some(operator) = operator {
        let lhs = lhs.expect("expected lhs");
        let rhs = rhs.expect("expected rhs");
        Ok(ast::Expression::Operator { lhs, operator, rhs })
    } else {
        Ok(ast::Expression::Term(lhs.expect("expected term")))
    }
}

fn parse_expression_term(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::ExpressionTerm, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        parser::Rule::literal => Ok(ast::ExpressionTerm::Literal(parse_literal(inner_pair)?)),
        parser::Rule::identifier => Ok(ast::ExpressionTerm::Identifier(parse_identifier(
            inner_pair,
        )?)),
        parser::Rule::expression => Ok(ast::ExpressionTerm::Expression(Box::new(
            parse_expression(inner_pair)?,
        ))),
        parser::Rule::function_call => Ok(ast::ExpressionTerm::FunctionCall(parse_function_call(
            inner_pair,
        )?)),
        rule => unimplemented!("term expression rule: {rule:?}"),
    }
}

fn parse_identifier(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Identifier, ParseError> {
    Ok(ast::Identifier(pair.as_str().trim().to_string()))
}

fn parse_operator(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Operator, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        parser::Rule::addition_operator => Ok(ast::Operator::Add),
        parser::Rule::multiplication_operator => Ok(ast::Operator::Multiply),
        rule => unimplemented!("operator rule: {rule:?}"),
    }
}

fn parse_literal(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Literal, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        parser::Rule::number => {
            let number = inner_pair.as_str().parse().expect("expected number");
            Ok(ast::Literal::Number(number))
        }
        parser::Rule::symbol => Ok(ast::Literal::Symbol(inner_pair.as_str().trim().to_string())),
        rule => unimplemented!("literal rule: {rule:?}"),
    }
}

fn parse_function_call(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::FunctionCall, ParseError> {
    let mut identifier = None;
    let mut arguments = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::identifier => {
                identifier = Some(parse_identifier(inner_pair)?);
            }
            parser::Rule::argument => {
                arguments.push(parse_argument(inner_pair)?);
            }
            rule => unimplemented!("function call rule: {rule:?}"),
        }
    }

    Ok(ast::FunctionCall {
        identifier: identifier.expect("expected function name"),
        arguments,
    })
}

fn parse_argument(
    pair: pest::iterators::Pair<parser::Rule>,
) -> std::result::Result<ast::Argument, ParseError> {
    let mut identifier = None;
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            parser::Rule::identifier => {
                identifier = Some(parse_identifier(inner_pair)?);
            }
            parser::Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            rule => unimplemented!("argument rule: {rule:?}"),
        }
    }

    Ok(ast::Argument {
        identifier: identifier.expect("expected argument name"),
        value: value.expect("expected argument value"),
    })
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
                            ast::Literal::Symbol(":symbol".to_string())
                        ))
                    }]
                }
            },
            actual
        );
    }
}
