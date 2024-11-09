use std::result::Result;

use ast::{Expression, Operator};
use parser::{BloxParser, Rule};
use pest::{pratt_parser::PrattParser, Parser};

use lazy_static::lazy_static;
use rust_decimal::Decimal;

pub mod ast;
pub mod parser;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    PestError(pest::error::Error<Rule>),
    DecimalError(rust_decimal::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::PestError(err) => write!(f, "{}", err),
            ParseError::DecimalError(err) => write!(f, "{}", err),
        }
    }
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        ParseError::PestError(err)
    }
}

impl From<rust_decimal::Error> for ParseError {
    fn from(err: rust_decimal::Error) -> Self {
        ParseError::DecimalError(err)
    }
}

pub fn parse(code: &str) -> Result<ast::Program, ParseError> {
    let mut result = BloxParser::parse(Rule::program, code)?;
    assert_eq!(result.len(), 1);

    let Some(pair) = result.next() else {
        panic!("expected a single program rule, found none");
    };

    Ok(parse_program(pair)?)
}

fn parse_program(pair: pest::iterators::Pair<Rule>) -> Result<ast::Program, ParseError> {
    let mut block = None;
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::block => {
                block = Some(parse_block(inner_pair)?);
            }
            Rule::EOI => {}
            rule => unimplemented!("program rule: {rule:?}"),
        }
    }

    Ok(ast::Program(block.expect("expected block")))
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<ast::Block, ParseError> {
    let mut statements = vec![];
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::statement => {
                statements.push(parse_statement(inner_pair)?);
            }
            rule => unimplemented!("block rule: {rule:?}"),
        }
    }

    Ok(ast::Block(statements))
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<ast::Statement, ParseError> {
    let mut result = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::definition => {
                let definition = parse_definition(inner_pair)?;
                result = Some(ast::Statement::Definition(definition));
            }
            Rule::binding => {
                let (lhs, rhs) = parse_binding(inner_pair)?;
                result = Some(ast::Statement::Binding(lhs, rhs));
            }
            Rule::import => {
                let import = parse_import(inner_pair)?;
                result = Some(ast::Statement::Import(import));
            }
            Rule::expression => {
                let expression = parse_expression(inner_pair)?;
                result = Some(ast::Statement::Expression(expression));
            }
            rule => unimplemented!("statement rule: {rule:?}"),
        }
    }

    Ok(result.expect("expected statement"))
}

fn parse_definition(pair: pest::iterators::Pair<Rule>) -> Result<ast::Definition, ParseError> {
    let mut name = None;
    let mut parameters = vec![];
    let mut body = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier if name == None => {
                name = Some(parse_identifier(inner_pair)?);
            }
            Rule::identifier => {
                parameters.push(ast::Parameter(parse_identifier(inner_pair)?));
            }
            Rule::block => {
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

fn parse_binding(
    pair: pest::iterators::Pair<Rule>,
) -> Result<(ast::Identifier, ast::Expression), ParseError> {
    let mut lhs = None;
    let mut rhs = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                lhs = Some(parse_identifier(inner_pair)?);
            }
            Rule::expression => {
                rhs = Some(parse_expression(inner_pair)?);
            }
            rule => unimplemented!("binding rule: {rule:?}"),
        }
    }

    Ok((lhs.expect("expected lhs"), rhs.expect("expected rhs")))
}

pub fn parse_import(pair: pest::iterators::Pair<Rule>) -> Result<ast::Import, ParseError> {
    let mut symbols = vec![];
    let mut path = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::imported_symbol => {
                symbols.push(parse_imported_symbol(inner_pair)?);
            }
            Rule::string => {
                path = Some(parse_string(inner_pair)?);
            }
            rule => unimplemented!("import rule: {rule:?}"),
        }
    }

    let path = path.expect("expected path");

    Ok(ast::Import(symbols, path))
}

fn parse_imported_symbol(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ast::ImportedSymbol, ParseError> {
    let mut name = None;
    let mut alias = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                if name == None {
                    name = Some(parse_identifier(inner_pair)?);
                } else {
                    alias = Some(parse_identifier(inner_pair)?);
                }
            }
            rule => unimplemented!("imported symbol rule: {rule:?}"),
        }
    }

    let name = name.expect("expected name");

    Ok(ast::ImportedSymbol(name, alias))
}

pub fn parse_expression_string(code: &str) -> Result<ast::Expression, ParseError> {
    let mut result = BloxParser::parse(Rule::expression, code)?;
    assert_eq!(result.len(), 1);

    let Some(pair) = result.next() else {
        unreachable!("expected a single expression rule, found none");
    };

    parse_expression(pair)
}

lazy_static! {
    static ref EXPRESSION_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(concatenate, Left))
            .op(Op::infix(multiply, Left))
    };
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::Expression, ParseError> {
    EXPRESSION_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::expression_term => Ok(ast::Expression::Term(parse_expression_term(primary)?)),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Operator::Add,
                Rule::multiply => Operator::Multiply,
                Rule::concatenate => Operator::Concatenate,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Ok(Expression::Operator(Box::new(lhs?), op, Box::new(rhs?)))
        })
        .parse(pair.into_inner())
}

fn parse_expression_term(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ast::ExpressionTerm, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        Rule::literal => Ok(ast::ExpressionTerm::Literal(parse_literal(inner_pair)?)),
        Rule::identifier => Ok(ast::ExpressionTerm::Identifier(parse_identifier(
            inner_pair,
        )?)),
        Rule::expression => Ok(ast::ExpressionTerm::Expression(Box::new(parse_expression(
            inner_pair,
        )?))),
        Rule::function_call => Ok(ast::ExpressionTerm::FunctionCall(parse_function_call(
            inner_pair,
        )?)),
        Rule::array => Ok(ast::ExpressionTerm::Array(parse_array(inner_pair)?)),
        Rule::array_index => Ok(ast::ExpressionTerm::ArrayIndex(parse_array_index(
            inner_pair,
        )?)),
        Rule::object => Ok(ast::ExpressionTerm::Object(parse_object(inner_pair)?)),
        Rule::object_index => Ok(ast::ExpressionTerm::ObjectIndex(parse_object_index(
            inner_pair,
        )?)),
        Rule::if_expression => Ok(ast::ExpressionTerm::If(parse_if_expression(inner_pair)?)),
        rule => unimplemented!("term expression rule: {rule:?}"),
    }
}

fn parse_identifier(pair: pest::iterators::Pair<Rule>) -> Result<ast::Identifier, ParseError> {
    Ok(ast::Identifier(pair.as_str().trim().to_string()))
}

fn parse_literal(pair: pest::iterators::Pair<Rule>) -> Result<ast::Literal, ParseError> {
    let inner_pair = pair.into_inner().next().expect("expected inner pair");
    match inner_pair.as_rule() {
        Rule::number => {
            let number = Decimal::from_str_radix(inner_pair.as_str(), 10)?;
            Ok(ast::Literal::Number(number))
        }
        Rule::string => {
            let s = parse_string(inner_pair)?;
            Ok(ast::Literal::String(s))
        }
        Rule::symbol => Ok(ast::Literal::Symbol(inner_pair.as_str().trim().to_string())),
        rule => unimplemented!("literal rule: {rule:?}"),
    }
}

fn parse_string(pair: pest::iterators::Pair<Rule>) -> Result<String, ParseError> {
    let s = pair.as_str();

    // strip off the quotes at either end
    let s = s.get(1..s.len() - 1).expect("expected inner pair");

    Ok(s.to_string())
}

fn parse_function_call(pair: pest::iterators::Pair<Rule>) -> Result<ast::FunctionCall, ParseError> {
    let mut identifier = None;
    let mut arguments = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                identifier = Some(parse_identifier(inner_pair)?);
            }
            Rule::argument => {
                arguments.push(parse_argument(inner_pair)?);
            }
            rule => unimplemented!("function call rule: {rule:?}"),
        }
    }

    Ok(ast::FunctionCall(
        identifier.expect("expected function name"),
        arguments,
    ))
}

fn parse_array(pair: pest::iterators::Pair<Rule>) -> Result<ast::Array, ParseError> {
    let mut members = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                members.push(parse_expression(inner_pair)?);
            }
            rule => unimplemented!("array rule: {rule:?}"),
        }
    }

    Ok(ast::Array(members))
}

fn parse_array_index(pair: pest::iterators::Pair<Rule>) -> Result<ast::ArrayIndex, ParseError> {
    let mut array = None;
    let mut index = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier if array == None => {
                array = Some(ast::ExpressionTerm::Identifier(parse_identifier(
                    inner_pair,
                )?));
            }
            Rule::function_call if array == None => {
                array = Some(ast::ExpressionTerm::FunctionCall(parse_function_call(
                    inner_pair,
                )?));
            }
            Rule::array if index == None => {
                array = Some(ast::ExpressionTerm::Array(parse_array(inner_pair)?));
            }
            Rule::expression if index == None => {
                index = Some(parse_expression(inner_pair)?);
            }
            rule => unreachable!("unexpected {rule:?}"),
        }
    }

    Ok(ast::ArrayIndex {
        array: Box::new(array.expect("expected array name")),
        index: Box::new(index.expect("expected array index")),
    })
}

fn parse_object(pair: pest::iterators::Pair<Rule>) -> Result<ast::Object, ParseError> {
    let mut members = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::object_member => {
                members.push(parse_object_member(inner_pair)?);
            }
            rule => unreachable!("object rule: {rule:?}"),
        }
    }

    Ok(ast::Object(members))
}

fn parse_object_member(
    pair: pest::iterators::Pair<Rule>,
) -> Result<(String, Expression), ParseError> {
    let mut key = None;
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                key = Some(inner_pair.as_str().trim().to_string());
            }
            Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            rule => unreachable!("object member rule: {rule:?}"),
        }
    }

    Ok((
        key.expect("expected object key"),
        value.expect("expected object value"),
    ))
}

fn parse_object_index(pair: pest::iterators::Pair<Rule>) -> Result<ast::ObjectIndex, ParseError> {
    let mut object = None;
    let mut key = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier if object == None => {
                object = Some(ast::ExpressionTerm::Identifier(parse_identifier(
                    inner_pair,
                )?));
            }
            Rule::function_call if object == None => {
                object = Some(ast::ExpressionTerm::FunctionCall(parse_function_call(
                    inner_pair,
                )?));
            }
            Rule::object if object == None => {
                object = Some(ast::ExpressionTerm::Object(parse_object(inner_pair)?));
            }
            Rule::identifier if key == None => {
                key = Some(inner_pair.as_str().trim().to_string());
            }
            rule => unreachable!("unexpected {rule:?}"),
        }
    }

    Ok(ast::ObjectIndex {
        object: Box::new(object.expect("expected object name")),
        key: key.expect("expected object key"),
    })
}

fn parse_if_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::If, ParseError> {
    let mut condition = None;
    let mut then_branch = None;
    let mut else_branch = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                condition = Some(parse_expression(inner_pair)?);
            }
            Rule::block if then_branch == None => {
                then_branch = Some(parse_block(inner_pair)?);
            }
            Rule::block => {
                else_branch = Some(parse_block(inner_pair)?);
            }
            rule => unreachable!("if expression rule: {rule:?}"),
        }
    }

    Ok(ast::If {
        condition: Box::new(condition.expect("expected condition")),
        then_branch: Box::new(then_branch.expect("expected then block")),
        else_branch: else_branch.map(Box::new),
    })
}

fn parse_argument(pair: pest::iterators::Pair<Rule>) -> Result<ast::Argument, ParseError> {
    let mut identifier = None;
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                identifier = Some(parse_identifier(inner_pair)?);
            }
            Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            rule => unreachable!("argument rule: {rule:?}"),
        }
    }

    Ok(ast::Argument(
        identifier.expect("expected argument name"),
        value.expect("expected argument value"),
    ))
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::parse;

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
                ast::Expression::Operator(
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
                ast::Expression::Operator(
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Expression(
                        Box::new(ast::Expression::Operator(
                            Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                                ast::Literal::Number(1.into())
                            ))),
                            ast::Operator::Multiply,
                            Box::new(ast::Expression::Term(ast::ExpressionTerm::Literal(
                                ast::Literal::Number(2.into())
                            )))
                        ))
                    ))),
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
                    ":symbol".to_string()
                )))
            )])),
            actual
        );
    }
}
