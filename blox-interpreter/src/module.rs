use std::{collections::BTreeMap, sync::Arc};

use blox_language::ast;

use crate::{program::evaluate_block, RuntimeError, Scope, Value};

pub fn load_module(path: &str) -> Result<Module, RuntimeError> {
    let filename = format!("{}.blox", path);
    let source = std::fs::read_to_string(&filename)
        .map_err(|_| RuntimeError::ModuleNotFound(filename.clone()))?;
    load_module_from_string(path, &source)
}

pub fn load_module_from_string(path: &str, source: &str) -> Result<Module, RuntimeError> {
    let parser = blox_language::Parser::new(source);
    let ast = parser.parse()?;
    let module = evalute_module(&path, ast)?;
    Ok(module)
}

pub fn evalute_module(path: &str, ast: ast::Program) -> Result<Module, RuntimeError> {
    let mut scope = Arc::new(Scope::default());
    evaluate_block(&ast.0, &mut scope)?;

    let module = Module::new(path.to_string(), scope.bindings.read().unwrap().clone());
    Ok(module)
}

#[derive(Debug, Clone, PartialEq)]
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
