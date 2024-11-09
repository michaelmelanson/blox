use blox_language::{ast, ParseError};

use crate::{module::Module, Value};

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    ParseError(ParseError),
    UndefinedVariable(String),
    InvalidOperands {
        lhs_expression: ast::Expression,
        lhs_value: Value,
        operator: ast::Operator,
        rhs_expression: ast::Expression,
        rhs_value: Value,
    },
    InvalidCondition {
        condition_expression: ast::Expression,
        condition_value: Value,
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
    ModuleNotFound(String),
    ExportNotFound(Module, ast::Identifier),
    DecimalConversionError(rust_decimal::Error),
}

impl std::error::Error for RuntimeError {}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::ParseError(error) => {
                write!(f, "runtime parse error: {}", error)
            }

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
            RuntimeError::InvalidCondition {
                condition_expression,
                condition_value,
            } => {
                write!(
                    f,
                    "invalid condition: {condition_expression} (={condition_value})"
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
            RuntimeError::ModuleNotFound(name) => {
                write!(f, "module not found: {name}")
            }
            RuntimeError::ExportNotFound(module, identifier) => {
                write!(f, "export not found in {}: {identifier}", module.path)
            }
            RuntimeError::DecimalConversionError(error) => {
                write!(f, "decimal conversion error: {error}")
            }
        }
    }
}

impl From<ParseError> for RuntimeError {
    fn from(error: ParseError) -> Self {
        RuntimeError::ParseError(error)
    }
}

impl From<rust_decimal::Error> for RuntimeError {
    fn from(error: rust_decimal::Error) -> Self {
        RuntimeError::DecimalConversionError(error)
    }
}
