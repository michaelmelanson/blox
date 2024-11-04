use blox_language::ast;

use crate::Value;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    UndefinedVariable(String),
    InvalidOperands {
        lhs_expression: ast::Expression,
        lhs_value: Value,
        operator: ast::Operator,
        rhs_expression: ast::Expression,
        rhs_value: Value,
    },
    InvalidArrayIndex {
        array_expression: ast::ExpressionTerm,
        array_value: Value,
        index_expression: ast::Expression,
        index_value: Value,
    },
    ArrayIndexOutOfBounds {
        array_expression: ast::ExpressionTerm,
        array_value: Value,
        index_expression: ast::Expression,
        index_value: Value,
    },
    NotAFunction {
        identifier: ast::Identifier,
        value: Value,
    },
    NotAnObject {
        object_expression: ast::ExpressionTerm,
        object_value: Value,
        key: String,
    },
    ObjectKeyNotFound {
        object_expression: ast::ExpressionTerm,
        object_value: Value,
        key: String,
    },
}

impl std::error::Error for RuntimeError {}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "undefined variable: {}", name)
            }

            RuntimeError::InvalidOperands {
                lhs_expression,
                lhs_value,
                operator,
                rhs_expression,
                rhs_value,
            } => {
                write!(
                    f,
                    "invalid operands: {operator} cannot be used for {lhs_expression} (={lhs_value}) and {rhs_expression} (={rhs_value})"
                )
            }
            RuntimeError::NotAFunction { identifier, value } => {
                write!(f, "{identifier} is not a function: {value}")
            }
            RuntimeError::InvalidArrayIndex {
                array_expression,
                array_value,
                index_expression,
                index_value,
            } => {
                write!(f, "invalid array index: {array_expression} (={array_value})[{index_expression} (={index_value})]")
            }
            RuntimeError::ArrayIndexOutOfBounds {
                array_expression,
                array_value,
                index_expression,
                index_value,
            } => {
                write!(f, "array index out of bounds: {array_expression} (={array_value})[{index_expression} (={index_value})]")
            }
            RuntimeError::NotAnObject {
                object_expression,
                object_value,
                key,
            } => {
                write!(
                    f,
                    "{object_expression} (={object_value}) is not an object: {key}"
                )
            }
            RuntimeError::ObjectKeyNotFound {
                object_expression,
                object_value,
                key,
            } => {
                write!(
                    f,
                    "object key not found: {object_expression} (={object_value}).{key}"
                )
            }
        }
    }
}
