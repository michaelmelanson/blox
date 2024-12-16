use std::{collections::BTreeMap, sync::Arc};

use blox_language::ast;

use crate::{program::evaluate_block, RuntimeError, Scope, Value};

#[derive(Clone)]
pub struct EvaluationContext {
    pub import_base_dir: String,
    pub scope: Arc<Scope>,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            import_base_dir: ".".to_string(),
            scope: Arc::new(Scope::default()),
        }
    }
}

impl EvaluationContext {
    pub fn new(import_base_dir: impl ToString, scope: &Arc<Scope>) -> Self {
        Self {
            import_base_dir: import_base_dir.to_string(),
            scope: scope.clone(),
        }
    }

    pub fn child(&self) -> Self {
        Self {
            import_base_dir: self.import_base_dir.clone(),
            scope: self.scope.child(),
        }
    }

    pub fn child_with_scope(&self, call_scope: Arc<Scope>) -> Self {
        Self {
            import_base_dir: self.import_base_dir.clone(),
            scope: call_scope,
        }
    }
}

pub fn load_module(path: &str, context: &EvaluationContext) -> Result<Module, RuntimeError> {
    let filename = format!("{}/{}.blox", context.import_base_dir, path);
    // resolve the filename
    let filename = std::fs::canonicalize(&filename)
        .map_err(|err| RuntimeError::ModuleNotFound(err.to_string()))?
        .to_str()
        .unwrap()
        .to_string();

    let source = std::fs::read_to_string(&filename)
        .map_err(|_| RuntimeError::ModuleNotFound(filename.clone()))?;
    load_module_from_string(path, &source, context)
}

pub fn load_module_from_string(
    path: &str,
    source: &str,
    context: &EvaluationContext,
) -> Result<Module, RuntimeError> {
    let parser = blox_language::Parser::new(source);
    let ast = parser.parse()?;
    let module = evalute_module(&path, ast, context)?;
    Ok(module)
}

pub fn evalute_module(
    path: &str,
    ast: ast::Program,
    context: &EvaluationContext,
) -> Result<Module, RuntimeError> {
    let mut context = context.child();
    evaluate_block(&ast.0, &mut context)?;

    let module = Module::new(
        path.to_string(),
        context.scope.bindings.read().unwrap().clone(),
    );
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
