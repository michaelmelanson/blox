use blox_language::ast;

use crate::Value;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    UndefinedVariable(String),
    InvalidOperands {
        lhs_expression: ast::ExpressionTerm,
        lhs_value: Value,
        operator: ast::Operator,
        rhs_expression: ast::ExpressionTerm,
        rhs_value: Value,
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
        }
    }
}
