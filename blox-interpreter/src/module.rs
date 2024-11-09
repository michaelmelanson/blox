use std::collections::BTreeMap;

use blox_language::ast;

use crate::{RuntimeError, Scope, Value};

pub fn load_module(path: &str) -> Result<Module, RuntimeError> {
    let filename = format!("{}.blox", path);
    let source = std::fs::read_to_string(&filename)
        .map_err(|_| RuntimeError::ModuleNotFound(filename.clone()))?;
    let ast = blox_language::parse(&source)?;
    let module = evalute_module(&path, ast);
    Ok(module)
}

pub fn evalute_module(path: &str, ast: ast::Program) -> Module {
    let mut scope = Scope::default();

    let block = ast.0;

    for statement in block.0 {
        match statement {
            ast::Statement::Definition(definition) => {
                let closure = scope.clone();
                let function = Value::Function(definition.clone(), closure);
                scope.insert_binding(&definition.name, function);
            }
            _ => unimplemented!(),
        }
    }

    Module::new(path.to_string(), scope.bindings)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    pub path: String,
    pub exports: BTreeMap<ast::Identifier, Value>,
}

impl Module {
    pub fn new(path: String, exports: BTreeMap<ast::Identifier, Value>) -> Self {
        Self { path, exports }
    }

    pub fn export(&self, name: &ast::Identifier) -> Result<&Value, RuntimeError> {
        self.exports
            .get(name)
            .ok_or_else(|| RuntimeError::ExportNotFound(self.clone(), name.clone()))
    }
}
