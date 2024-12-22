use rust_decimal::Decimal;
use tracing::trace;
use tree_sitter::Node;

use crate::{ast, error::ParseError, location::Location};

pub struct Parser<'a> {
    file: String,
    source: &'a str,
    tree: tree_sitter::Tree,
}

impl<'a> Parser<'a> {
    pub fn new(file: impl ToString, source: &'a str) -> Self {
        let mut ts_parser = tree_sitter::Parser::new();
        let language = tree_sitter_blox::LANGUAGE;
        ts_parser
            .set_language(&language.into())
            .expect("Error loading Blox parser");
        let tree = ts_parser.parse(source, None).unwrap();

        Parser {
            file: file.to_string(),
            source,
            tree,
        }
    }

    fn location(&self, node: Node<'_>) -> Location {
        Location {
            file: self.file.clone(),
            range: node.range(),
        }
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
        Ok(ast::Program {
            block,
            location: self.location(node),
        })
    }

    fn parse_block(&self, node: Node<'_>) -> Result<ast::Block, ParseError> {
        let mut statements = vec![];

        let mut cursor = node.walk();
        for child in node.children_by_field_name("statement", &mut cursor) {
            statements.push(self.parse_statement(child)?);
        }

        Ok(ast::Block {
            statements,
            location: self.location(node),
        })
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
        )?;

        let body = self.parse_block(
            node.child_by_field_name("body")
                .expect("definition has no body"),
        )?;

        let mut parameters = vec![];
        for child in node.children_by_field_name("parameter", &mut node.walk()) {
            parameters.push(ast::Parameter(self.parse_identifier(child)?));
        }

        Ok(ast::Definition {
            name: Some(name),
            parameters,
            body,
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

    pub fn parse_lambda(&self, node: Node<'_>) -> Result<ast::Definition, ParseError> {
        let mut parameters = vec![];
        for child in node.children_by_field_name("parameter", &mut node.walk()) {
            parameters.push(ast::Parameter(self.parse_identifier(child)?));
        }

        let body = self.parse_block(
            node.child_by_field_name("body")
                .expect("lambda has no body"),
        )?;

        Ok(ast::Definition {
            name: None,
            parameters,
            body,
        })
    }

    fn parse_expression(&self, node: Node<'_>) -> Result<ast::Expression, ParseError> {
        let result = match node.kind() {
            "binary_expression" => {
                let (lhs, operator, rhs) = self.parse_binary_expression(node)?;
                ast::Expression::BinaryExpression(lhs, operator, rhs)
            }
            "if_expression" => {
                ast::Expression::Term(ast::ExpressionTerm::If(self.parse_if_expression(node)?))
            }
            "array_slice" => ast::Expression::Term(ast::ExpressionTerm::ArraySlice(
                self.parse_array_slice(node)?,
            )),
            "array_index" => ast::Expression::Term(ast::ExpressionTerm::ArrayIndex(
                self.parse_array_index(node)?,
            )),
            "object_index" => ast::Expression::Term(ast::ExpressionTerm::ObjectIndex(
                self.parse_object_index(node)?,
            )),
            "method_call" => ast::Expression::Term(ast::ExpressionTerm::MethodCall(
                self.parse_method_call(node)?,
            )),
            "function_call" => ast::Expression::Term(ast::ExpressionTerm::FunctionCall(
                self.parse_function_call(node)?,
            )),
            "literal" => {
                ast::Expression::Term(ast::ExpressionTerm::Literal(self.parse_literal(node)?))
            }
            "identifier" => ast::Expression::Term(ast::ExpressionTerm::Identifier(
                self.parse_identifier(node)?,
            )),
            "array" => {
                return Ok(ast::Expression::Term(ast::ExpressionTerm::Array(
                    self.parse_array(node)?,
                )));
            }
            "object" => {
                ast::Expression::Term(ast::ExpressionTerm::Object(self.parse_object(node)?))
            }
            "group_term" => ast::Expression::Term(ast::ExpressionTerm::Expression(Box::new(
                self.parse_expression_container(node)?,
            ))),
            "lambda" => {
                ast::Expression::Term(ast::ExpressionTerm::Lambda(self.parse_lambda(node)?))
            }
            "group" => self.parse_expression(
                node.child_by_field_name("expression")
                    .expect("group without expression"),
            )?,

            kind => unimplemented!("expression kind: {kind}"),
        };

        Ok(result)
    }

    fn parse_binary_expression(
        &self,
        node: Node<'_>,
    ) -> Result<(Box<ast::Expression>, ast::Operator, Box<ast::Expression>), ParseError> {
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
        Ok(ast::Identifier {
            name: identifier.to_string(),
        })
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
                    let value = self.parse_symbol(child)?;
                    return Ok(ast::Literal::Symbol(value.to_string()));
                }
                _ => {}
            }
        }
        unreachable!()
    }

    fn parse_operator(&self, node: Node<'_>) -> Result<ast::Operator, ParseError> {
        match node.kind() {
            "not" => Ok(ast::Operator::Not),
            "negate" => Ok(ast::Operator::Negate),
            "multiply" => Ok(ast::Operator::Multiply),
            "divide" => Ok(ast::Operator::Divide),
            "concatenate" => Ok(ast::Operator::Concatenate),
            "add" => Ok(ast::Operator::Add),
            "subtract" => Ok(ast::Operator::Subtract),
            "equal" => Ok(ast::Operator::Equal),
            "not_equal" => Ok(ast::Operator::NotEqual),
            "greater_or_equal" => Ok(ast::Operator::GreaterOrEqual),
            "greater_than" => Ok(ast::Operator::GreaterThan),
            "less_or_equal" => Ok(ast::Operator::LessOrEqual),
            "less_than" => Ok(ast::Operator::LessThan),

            "assignment" => Ok(ast::Operator::Assignment),
            "append" => Ok(ast::Operator::Append),
            "pipe" => Ok(ast::Operator::Pipe),
            kind => unimplemented!("operator kind: {kind}"),
        }
    }

    fn parse_string(&self, node: Node<'_>) -> Result<String, ParseError> {
        let s = self.value(node.range());

        // strip off the quotes at either end
        let s = s.get(1..s.len() - 1).expect("expected inner pair");

        Ok(s.to_string())
    }

    fn parse_symbol(&self, node: Node<'_>) -> Result<String, ParseError> {
        let s = self.value(node.range());

        // strip off the colon off the start
        let s = s.get(1..s.len()).expect("expected inner pair");

        Ok(s.to_string())
    }

    fn parse_method_call(&self, node: Node<'_>) -> Result<ast::MethodCall, ParseError> {
        let base = Box::new(
            self.parse_expression(
                node.child_by_field_name("base")
                    .expect("method call without base"),
            )?,
        );

        let function = self.parse_identifier(
            node.child_by_field_name("function")
                .expect("method call without function"),
        )?;

        let mut arguments = vec![];
        for child in node.children_by_field_name("argument", &mut node.walk()) {
            arguments.push(self.parse_argument(child)?);
        }

        Ok(ast::MethodCall {
            base,
            function,
            arguments,
        })
    }

    fn parse_function_call(&self, node: Node<'_>) -> Result<ast::FunctionCall, ParseError> {
        let function = Box::new(
            self.parse_expression(
                node.child_by_field_name("function")
                    .expect("function call without callee"),
            )?,
        );

        let mut arguments = vec![];
        for child in node.children_by_field_name("argument", &mut node.walk()) {
            arguments.push(self.parse_argument(child)?);
        }

        Ok(ast::FunctionCall(function, arguments))
    }

    fn parse_array(&self, node: Node<'_>) -> Result<ast::Array, ParseError> {
        let mut members = vec![];

        let mut cursor = node.walk();
        for child in node.children_by_field_name("member", &mut cursor) {
            members.push(self.parse_expression(child)?);
        }

        Ok(ast::Array(members))
    }

    fn parse_array_slice(&self, node: Node<'_>) -> Result<ast::ArraySlice, ParseError> {
        let base = Box::new(
            self.parse_expression(
                node.child_by_field_name("base")
                    .expect("array slice with no base"),
            )?,
        );

        let start_node = node.child_by_field_name("start");
        let end_node = node.child_by_field_name("end");

        let start = match start_node {
            Some(node) => Some(Box::new(self.parse_expression(node)?)),
            None => None,
        };

        let end = match end_node {
            Some(node) => Some(Box::new(self.parse_expression(node)?)),
            None => None,
        };

        Ok(ast::ArraySlice { base, start, end })
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

    fn parse_object_member(&self, node: Node<'_>) -> Result<(String, ast::Expression), ParseError> {
        let key = self.parse_identifier(
            node.child_by_field_name("key")
                .expect("object member without key"),
        )?;

        let value = self.parse_expression(
            node.child_by_field_name("value")
                .expect("object member without value"),
        )?;

        Ok((key.name, value))
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

    fn parse_elseif_expression(
        &self,
        node: Node<'_>,
    ) -> Result<(ast::Expression, ast::Block), ParseError> {
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

    fn parse_else_expression(&self, node: Node<'_>) -> Result<ast::Block, ParseError> {
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
