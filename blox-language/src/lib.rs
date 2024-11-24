use std::result::Result;

use ast::{Block, Expression, Operator};

use rust_decimal::Decimal;
use tracing::trace;
use tree_sitter::Node;

pub mod ast;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    DecimalError(rust_decimal::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::DecimalError(err) => write!(f, "{}", err),
        }
    }
}

impl From<rust_decimal::Error> for ParseError {
    fn from(err: rust_decimal::Error) -> Self {
        ParseError::DecimalError(err)
    }
}

pub struct Parser<'a> {
    source: &'a str,
    tree: tree_sitter::Tree,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut ts_parser = tree_sitter::Parser::new();
        let language = tree_sitter_blox::LANGUAGE;
        ts_parser
            .set_language(&language.into())
            .expect("Error loading Blox parser");
        let tree = ts_parser.parse(source, None).unwrap();

        Parser { source, tree }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn parse(&self) -> Result<ast::Program, ParseError> {
        trace!(source = self.source);

        let root = self.tree.root_node();
        trace!(tree = root.to_sexp());

        let ast = self.parse_program(root)?;
        trace!(?ast);

        Ok(ast)
    }

    pub fn parse_as_expression(&self) -> Result<ast::Expression, ParseError> {
        let node = self
            .tree
            .root_node() // source file
            .child(0)
            .unwrap() // expression statement
            .child(0)
            .unwrap(); // expression

        self.parse_expression(node)
    }

    fn value(&self, range: tree_sitter::Range) -> &str {
        self.source
            .get(range.start_byte..range.end_byte)
            .expect("invalid range")
    }

    fn parse_program(&self, node: Node<'_>) -> Result<ast::Program, ParseError> {
        let block = self.parse_block(node)?;
        Ok(ast::Program(block))
    }

    fn parse_block(&self, node: Node<'_>) -> Result<ast::Block, ParseError> {
        let mut statements = vec![];

        let mut cursor = node.walk();
        for child in node.children_by_field_name("statement", &mut cursor) {
            statements.push(self.parse_statement(child)?);
        }

        Ok(ast::Block(statements))
    }

    fn parse_statement(&self, node: Node<'_>) -> Result<ast::Statement, ParseError> {
        match node.kind() {
            "definition" => {
                let definition = self.parse_definition(node)?;
                Ok(ast::Statement::Definition(definition))
            }
            "binding" => {
                let (lhs, rhs) = self.parse_binding(node)?;
                Ok(ast::Statement::Binding(lhs, rhs))
            }
            "import" => {
                let import = self.parse_import(node)?;
                Ok(ast::Statement::Import(import))
            }
            "expression_statement" => {
                let expression = self.parse_expression_container(node)?;
                Ok(ast::Statement::Expression(expression))
            }
            kind => unimplemented!("statement kind: {kind}"),
        }
    }

    fn parse_definition(&self, node: Node<'_>) -> Result<ast::Definition, ParseError> {
        let name = self.parse_identifier(
            node.child_by_field_name("name")
                .expect("definition has no name"),
        );

        let body = self.parse_block(
            node.child_by_field_name("body")
                .expect("definition has no body"),
        );

        let mut parameters = vec![];
        for child in node.children_by_field_name("parameter", &mut node.walk()) {
            parameters.push(ast::Parameter(self.parse_identifier(child)?));
        }

        Ok(ast::Definition {
            name: name.expect("expected name"),
            parameters,
            body: body.expect("expected body"),
        })
    }

    fn parse_binding(
        &self,
        node: Node<'_>,
    ) -> Result<(ast::Identifier, ast::Expression), ParseError> {
        let name = self.parse_identifier(
            node.child_by_field_name("name")
                .expect("binding has no name"),
        )?;

        let value = self.parse_expression(
            node.child_by_field_name("value")
                .expect("binding has no value"),
        )?;

        Ok((name, value))
    }

    pub fn parse_import(&self, node: Node<'_>) -> Result<ast::Import, ParseError> {
        let mut symbols = vec![];
        let mut path = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "imported_symbol" {
                symbols.push(self.parse_imported_symbol(child)?);
            } else if child.kind() == "string" {
                path = Some(self.parse_string(child)?);
            }
        }

        let path = path.expect("expected path");

        Ok(ast::Import(symbols, path))
    }

    fn parse_imported_symbol(&self, node: Node<'_>) -> Result<ast::ImportedSymbol, ParseError> {
        let name = self.parse_identifier(
            node.child_by_field_name("identifier")
                .expect("imported symbol has no identifier"),
        )?;

        let mut alias = None;

        if let Some(node) = node.child_by_field_name("alias") {
            alias = Some(self.parse_identifier(node)?);
        }

        Ok(ast::ImportedSymbol(name, alias))
    }

    pub fn parse_expression_container(
        &self,
        node: Node<'_>,
    ) -> Result<ast::Expression, ParseError> {
        let expression = self.parse_expression(node.child_by_field_name("expression").expect(
            &format!("expression statement has no expression: {}", node.to_sexp()),
        ))?;

        Ok(expression)
    }

    fn parse_expression(&self, node: Node<'_>) -> Result<ast::Expression, ParseError> {
        match node.kind() {
            "term" => Ok(Expression::Term(self.parse_expression_term(node)?)),
            "binary_expression" => {
                let (lhs, operator, rhs) = self.parse_binary_expression(node)?;
                Ok(Expression::BinaryExpression(lhs, operator, rhs))
            }
            kind => unimplemented!("expression kind: {kind}"),
        }
    }

    fn parse_expression_term(&self, node: Node<'_>) -> Result<ast::ExpressionTerm, ParseError> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_expression" => {
                    return Ok(ast::ExpressionTerm::If(self.parse_if_expression(child)?))
                }
                "array_index" => {
                    return Ok(ast::ExpressionTerm::ArrayIndex(
                        self.parse_array_index(child)?,
                    ))
                }
                "object_index" => {
                    return Ok(ast::ExpressionTerm::ObjectIndex(
                        self.parse_object_index(child)?,
                    ))
                }
                "function_call" => {
                    return Ok(ast::ExpressionTerm::FunctionCall(
                        self.parse_function_call(child)?,
                    ))
                }
                "literal" => return Ok(ast::ExpressionTerm::Literal(self.parse_literal(child)?)),
                "identifier" => {
                    return Ok(ast::ExpressionTerm::Identifier(
                        self.parse_identifier(child)?,
                    ))
                }
                "array" => return Ok(ast::ExpressionTerm::Array(self.parse_array(child)?)),
                "object" => return Ok(ast::ExpressionTerm::Object(self.parse_object(child)?)),
                "group_term" => {
                    return Ok(ast::ExpressionTerm::Expression(Box::new(
                        self.parse_expression_container(child)?,
                    )))
                }
                kind => unimplemented!("expression term kind: {kind}"),
            }
        }

        unreachable!();
    }

    fn parse_binary_expression(
        &self,
        node: Node<'_>,
    ) -> Result<(Box<Expression>, Operator, Box<Expression>), ParseError> {
        let lhs = self.parse_expression(
            node.child_by_field_name("lhs")
                .expect("binary expression has no left"),
        )?;

        let operator = self.parse_operator(
            node.child_by_field_name("operator")
                .expect("binary expression has no operator"),
        )?;

        let rhs = self.parse_expression(
            node.child_by_field_name("rhs")
                .expect("binary expression has no right"),
        )?;

        Ok((Box::new(lhs), operator, Box::new(rhs)))
    }

    fn parse_identifier(&self, node: Node<'_>) -> Result<ast::Identifier, ParseError> {
        let identifier = self.value(node.range());
        Ok(ast::Identifier(identifier.to_string()))
    }

    fn parse_literal(&self, node: Node<'_>) -> Result<ast::Literal, ParseError> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "boolean" => {
                    if child.child(0).unwrap().kind() == "boolean_true" {
                        return Ok(ast::Literal::Boolean(true));
                    } else {
                        return Ok(ast::Literal::Boolean(false));
                    }
                }
                "number" => {
                    let value = self.value(child.range());
                    let number = Decimal::from_str_radix(value, 10)?;
                    return Ok(ast::Literal::Number(number));
                }
                "string" => {
                    let s = self.parse_string(child)?;
                    return Ok(ast::Literal::String(s));
                }
                "symbol" => {
                    let value = self.value(child.range());
                    return Ok(ast::Literal::Symbol(value.to_string()));
                }
                _ => {}
            }
        }
        unreachable!()
    }

    fn parse_operator(&self, node: Node<'_>) -> Result<Operator, ParseError> {
        match node.kind() {
            "not" => Ok(Operator::Not),
            "negate" => Ok(Operator::Negate),
            "multiply" => Ok(Operator::Multiply),
            "divide" => Ok(Operator::Divide),
            "concatenate" => Ok(Operator::Concatenate),
            "add" => Ok(Operator::Add),
            "subtract" => Ok(Operator::Subtract),
            "equal" => Ok(Operator::Equal),
            "not_equal" => Ok(Operator::NotEqual),
            "greater_or_equal" => Ok(Operator::GreaterOrEqual),
            "greater_than" => Ok(Operator::GreaterThan),
            "less_or_equal" => Ok(Operator::LessOrEqual),
            "less_than" => Ok(Operator::LessThan),
            kind => unimplemented!("operator kind: {kind}"),
        }
    }

    fn parse_string(&self, node: Node<'_>) -> Result<String, ParseError> {
        let s = self.value(node.range());

        // strip off the quotes at either end
        let s = s.get(1..s.len() - 1).expect("expected inner pair");

        Ok(s.to_string())
    }

    fn parse_function_call(&self, node: Node<'_>) -> Result<ast::FunctionCall, ParseError> {
        let identifier = self.parse_identifier(
            node.child_by_field_name("name")
                .expect("function call without name"),
        )?;

        let mut arguments = vec![];
        for child in node.children_by_field_name("argument", &mut node.walk()) {
            arguments.push(self.parse_argument(child)?);
        }

        Ok(ast::FunctionCall(identifier, arguments))
    }

    fn parse_array(&self, node: Node<'_>) -> Result<ast::Array, ParseError> {
        let mut members = vec![];

        let mut cursor = node.walk();
        for child in node.children_by_field_name("member", &mut cursor) {
            members.push(self.parse_expression(child)?);
        }

        Ok(ast::Array(members))
    }

    fn parse_array_index(&self, node: Node<'_>) -> Result<ast::ArrayIndex, ParseError> {
        let base = self.parse_expression(
            node.child_by_field_name("base")
                .expect("array index with no base"),
        )?;

        let index = self.parse_expression(
            node.child_by_field_name("index")
                .expect("array index with no index"),
        )?;

        Ok(ast::ArrayIndex {
            base: Box::new(base),
            index: Box::new(index),
        })
    }

    fn parse_object(&self, node: Node<'_>) -> Result<ast::Object, ParseError> {
        let mut members = vec![];

        let mut cursor = node.walk();
        for child in node.children_by_field_name("member", &mut cursor) {
            members.push(self.parse_object_member(child)?);
        }

        Ok(ast::Object(members))
    }

    fn parse_object_member(&self, node: Node<'_>) -> Result<(String, Expression), ParseError> {
        let key = self.parse_identifier(
            node.child_by_field_name("key")
                .expect("object member without key"),
        )?;

        let value = self.parse_expression(
            node.child_by_field_name("value")
                .expect("object member without value"),
        )?;

        Ok((key.0, value))
    }

    fn parse_object_index(&self, node: Node<'_>) -> Result<ast::ObjectIndex, ParseError> {
        let base = self.parse_expression(
            node.child_by_field_name("base")
                .expect("object index without base"),
        )?;

        let index = self.parse_identifier(
            node.child_by_field_name("index")
                .expect("object index without index"),
        )?;

        Ok(ast::ObjectIndex {
            base: Box::new(base),
            index,
        })
    }

    fn parse_if_expression(&self, node: Node<'_>) -> Result<ast::If, ParseError> {
        let condition = self.parse_expression(
            node.child_by_field_name("condition")
                .expect("if expression without condition"),
        )?;

        let body = self.parse_block(
            node.child_by_field_name("body")
                .expect("if expression without body"),
        )?;

        let mut elseif_branches = vec![];
        let mut cursor = node.walk();
        for child in node.children_by_field_name("elseif", &mut cursor) {
            elseif_branches.push(self.parse_elseif_expression(child)?);
        }

        let mut else_branch = None;
        if let Some(else_node) = node.child_by_field_name("else") {
            else_branch = Some(self.parse_else_expression(else_node)?);
        }

        Ok(ast::If {
            condition: Box::new(condition),
            body,
            elseif_branches,
            else_branch,
        })
    }

    fn parse_elseif_expression(&self, node: Node<'_>) -> Result<(Expression, Block), ParseError> {
        let condition = self.parse_expression(
            node.child_by_field_name("condition")
                .expect("else if without condition"),
        )?;

        let body = self.parse_block(
            node.child_by_field_name("body")
                .expect("else if without body"),
        )?;

        Ok((condition, body))
    }

    fn parse_else_expression(&self, node: Node<'_>) -> Result<Block, ParseError> {
        let block =
            self.parse_block(node.child_by_field_name("body").expect("else without body"))?;

        Ok(block)
    }

    fn parse_argument(&self, node: Node<'_>) -> Result<ast::Argument, ParseError> {
        let name = self.parse_identifier(
            node.child_by_field_name("name")
                .expect("argument without name"),
        )?;

        let value = self.parse_expression(
            node.child_by_field_name("value")
                .expect("argument without value"),
        )?;

        Ok(ast::Argument(name, value))
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast, ParseError, Parser};

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
                    Box::new(ast::Expression::Term(ast::ExpressionTerm::Expression(
                        Box::new(ast::Expression::BinaryExpression(
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
