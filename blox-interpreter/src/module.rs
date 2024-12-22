use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use blox_language::ast;

use crate::{program::evaluate_block, RuntimeError, Scope, Value};

#[derive(Clone)]
pub struct EvaluationContext {
    pub import_base_dir: String,
    pub scope: Arc<Scope>,
    pub import_cache: Arc<RwLock<BTreeMap<String, Module>>>,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            import_base_dir: ".".to_string(),
            scope: Arc::new(Scope::default()),
            import_cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl EvaluationContext {
    pub fn new(
        import_base_dir: impl ToString,
        scope: Arc<Scope>,
        import_cache: Arc<RwLock<BTreeMap<String, Module>>>,
    ) -> Self {
        Self {
            import_base_dir: import_base_dir.to_string(),
            scope,
            import_cache,
        }
    }

    pub fn child(&self) -> Self {
        Self {
            import_base_dir: self.import_base_dir.clone(),
            scope: self.scope.child(),
            import_cache: self.import_cache.clone(),
        }
    }

    pub fn child_with_scope(&self, call_scope: Arc<Scope>) -> Self {
        Self {
            import_base_dir: self.import_base_dir.clone(),
            import_cache: self.import_cache.clone(),
            scope: call_scope,
        }
    }
}

// standard library modules statically included in the binary
// so that they can be loaded without reading from the filesystem
const STDLIB: [(&'static str, &'static str); 3] = [
    ("stdlib/math", include_str!("../../stdlib/math.blox")),
    ("stdlib/list", include_str!("../../stdlib/list.blox")),
    (
        "stdlib/database",
        include_str!("../../stdlib/database.blox"),
    ),
];

pub fn load_stdlib(context: &mut EvaluationContext) {
    // load the standard library
    for (path, source) in STDLIB.iter() {
        let module = load_module_from_string(path, source, &context)
            .expect("failed to load stdlib module {path}");

        context
            .import_cache
            .write()
            .unwrap()
            .insert(path.to_string(), module);
    }
}

pub fn load_module(
    import_path: &str,
    context: &mut EvaluationContext,
) -> Result<Module, RuntimeError> {
    let mut import_cache = context.import_cache.write().expect("import cache poisoned");

    if let Some(module) = import_cache.get(import_path) {
        return Ok(module.clone());
    }

    let filename = format!("{}/{}.blox", context.import_base_dir, import_path);

    // resolve the filename
    let filename = std::fs::canonicalize(&filename)
        .map_err(|err| RuntimeError::ModuleNotFound(err.to_string()))?
        .to_str()
        .unwrap()
        .to_string();

    let source = std::fs::read_to_string(&filename)
        .map_err(|_| RuntimeError::ModuleNotFound(filename.clone()))?;
    let module = load_module_from_string(import_path, &source, context)?;

    import_cache.insert(import_path.to_string(), module.clone());

    Ok(module)
}

pub fn load_module_from_string(
    path: &str,
    source: &str,
    context: &EvaluationContext,
) -> Result<Module, RuntimeError> {
    let parser = blox_language::parser::Parser::new(path, source);
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
    evaluate_block(&ast.block, &mut context)?;

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
