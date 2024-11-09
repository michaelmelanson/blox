use rust_decimal::Decimal;
use std::collections::BTreeMap;

use blox_language::ast;

use crate::{module::Module, Scope};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Void,
    Number(Decimal),
    String(String),
    Symbol(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),

    Function(ast::Definition, Scope),
    Module(Module),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "(void)"),
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "'{string}'",),
            Value::Symbol(symbol) => write!(f, ":{symbol}",),
            Value::Function(definition, _scope) => write!(
                f,
                "<function: {}/{}>",
                definition.name,
                definition.parameters.len()
            ),
            Value::Array(values) => {
                write!(f, "[")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(fields) => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, value)?;
                }
                write!(f, "}}")
            }
            Value::Module(module) => write!(f, "<module: {}>", module.path),
        }
    }
}
