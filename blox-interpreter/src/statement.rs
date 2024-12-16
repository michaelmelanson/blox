use blox_language::ast;

use crate::{
    expression::evaluate_expression,
    module::{load_module, EvaluationContext},
    value::Function,
    RuntimeError, Value,
};

pub fn execute_statement(
    statement: &ast::Statement,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    match statement {
        ast::Statement::Expression(expression) => {
            let value = evaluate_expression(expression, context)?;
            Ok(value)
        }
        ast::Statement::Binding(lhs, rhs) => {
            let value = evaluate_expression(rhs, context)?;
            context.scope.insert_binding(lhs, value.clone());
            Ok(value)
        }
        ast::Statement::Definition(definition) => {
            let closure = context.scope.child();

            let function = Value::Function(Function {
                definition: definition.clone(),
                closure: closure.clone(),
            });

            if let Some(name) = &definition.name {
                context.scope.insert_binding(name, function.clone());
            }

            Ok(function)
        }
        ast::Statement::Import(import) => {
            let module = load_module(&import.1, context)?;

            for symbol in &import.0 {
                let value = module.export(&symbol.0)?;
                let name = if symbol.1 == None {
                    symbol.0.clone()
                } else {
                    symbol.1.clone().unwrap()
                };

                context.scope.insert_binding(&name, value.clone());
            }

            Ok(Value::Module(module))
        }
    }
}
