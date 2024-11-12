use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use blox_language::ast;

use crate::{value::Value, RuntimeError};

#[derive(Default, Debug)]
pub struct Scope {
    pub parent: Option<Arc<Scope>>,
    pub bindings: RwLock<BTreeMap<ast::Identifier, Value>>,
}

impl Clone for Scope {
    fn clone(&self) -> Self {
        let bindings = self.bindings.read().expect("bindings poisoned in clone");

        Scope {
            parent: self.parent.clone(),
            bindings: RwLock::new(bindings.clone()),
        }
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        if self.parent != other.parent {
            return false;
        }

        let Ok(lhs) = self.bindings.read() else {
            panic!("binding is poisoned in eq")
        };

        let Ok(rhs) = other.bindings.read() else {
            panic!("binding is poisoned in eq")
        };

        &*lhs == &*rhs
    }
}

impl Scope {
    pub fn child(self: &Arc<Scope>) -> Arc<Self> {
        Arc::new(Scope {
            parent: Some(self.clone()),
            bindings: RwLock::new(BTreeMap::new()),
        })
    }

    pub fn insert_binding(&self, name: &ast::Identifier, value: Value) {
        let mut bindings = self.bindings.write().unwrap();

        bindings.insert(name.clone(), value);
    }

    pub fn get_binding(&self, name: &ast::Identifier) -> Result<Value, RuntimeError> {
        let mut scope = self;
        loop {
            let bindings = scope.bindings.read().unwrap();
            match bindings.get(name) {
                Some(value) => return Ok(value.clone()),
                None => match &scope.parent {
                    Some(parent) => scope = parent,
                    None => return Err(RuntimeError::UndefinedVariable(name.to_string())),
                },
            }
        }
    }
}
