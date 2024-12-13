use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{atomic::AtomicUsize, Arc},
};

use blox_language::ast::{self, Identifier};

use crate::{module::Module, RuntimeError, Scope};

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
    Intrinsic(Intrinsic),
}

impl Value {
    pub fn to_display_string(&self) -> String {
        match self {
            Value::String(v) => v.clone(),
            _ => self.to_string(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "(void)"),
            Value::Boolean(bool) => write!(f, "{bool}"),
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "{string}"),
            Value::Symbol(symbol) => write!(f, ":{symbol}"),
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
            Value::Intrinsic(intrinsic) => write!(f, "{intrinsic}"),
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
        if let Some(name) = &self.definition.name {
            write!(
                f,
                "<function {}/{}>",
                name,
                self.definition.parameters.len()
            )
        } else {
            write!(f, "<lambda/{}>", self.definition.parameters.len())
        }
    }
}

pub type IntrinsicFn =
    dyn Fn(HashMap<Identifier, Value>) -> Result<Value, RuntimeError> + Send + Sync;

#[derive(Clone)]
pub struct Intrinsic {
    pub id: usize,
    pub name: String,
    pub function: Arc<IntrinsicFn>,
}

impl Intrinsic {
    pub fn new(name: &str, function: Arc<IntrinsicFn>) -> Self {
        Self {
            id: Self::next_id(),
            name: name.to_string(),
            function,
        }
    }

    fn next_id() -> usize {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        id
    }
}

impl std::fmt::Debug for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Intrinsic")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<intrinsic: {}>", self.name)
    }
}

impl PartialEq for Intrinsic {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
