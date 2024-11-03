use std::collections::HashMap;

use blox_language::ast;

use crate::{value::Value, RuntimeError};

#[derive(Default, Debug)]
pub struct Scope {
    pub bindings: HashMap<ast::Identifier, Value>,
}

impl Scope {
    pub fn child(&self) -> Self {
        Scope {
            bindings: self.bindings.clone(),
        }
    }

    pub fn insert_binding(&mut self, name: &ast::Identifier, value: Value) {
        self.bindings.insert(name.clone(), value);
    }

    pub fn get_binding(&self, name: &str) -> Result<&Value, RuntimeError> {
        self.bindings
            .get(&ast::Identifier(name.to_string()))
            .ok_or(RuntimeError::UndefinedVariable(name.to_string()))
    }
}
