use rust_decimal::Decimal;
use std::{collections::BTreeMap, sync::Arc};

use blox_language::ast;

use crate::{module::Module, Scope};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Boolean(bool),
    Number(Decimal),
    String(String),
    Symbol(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Function(Function),
    Module(Module),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "(void)"),
            Value::Boolean(bool) => write!(f, "{bool}"),
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "'{string}'",),
            Value::Symbol(symbol) => write!(f, ":{symbol}",),
            Value::Function(function) => write!(f, "{function}"),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub definition: ast::Definition,
    pub closure: Arc<Scope>,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<function {}/{}>",
            self.definition.name,
            self.definition.parameters.len()
        )
    }
}
