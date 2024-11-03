use std::rc::Rc;

use blox_language::ast;

use crate::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Void,
    Number(i64),
    String(String),
    Symbol(String),
    Function(ast::Definition, Scope),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Value::Void => "(void)".to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => format!("'{string}'"),
            Value::Symbol(symbol) => format!(":{}", symbol),
            Value::Function(definition, _scope) => format!(
                "<function: {}/{}>",
                definition.name,
                definition.parameters.len()
            ),
        };

        write!(f, "{}", s)
    }
}
