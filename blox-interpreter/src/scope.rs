use std::collections::HashMap;

use blox_language::ast;

use crate::value::Value;

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

    pub fn insert_binding(&mut self, name: String, value: Value) {
        self.bindings.insert(ast::Identifier(name), value);
    }
}
