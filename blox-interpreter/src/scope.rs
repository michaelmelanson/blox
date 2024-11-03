use std::collections::{BTreeMap, HashMap};

use blox_language::ast;

use crate::{value::Value, RuntimeError};

#[derive(Default, Debug, Eq, PartialEq, Clone, Hash)]
pub struct Scope {
    pub bindings: BTreeMap<ast::Identifier, Value>,
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

    pub fn get_binding(&self, name: &ast::Identifier) -> Result<&Value, RuntimeError> {
        self.bindings
            .get(&name)
            .ok_or(RuntimeError::UndefinedVariable(name.to_string()))
    }
}
