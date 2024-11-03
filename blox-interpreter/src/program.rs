use blox_language::ast;

use crate::{statement::execute_statement, RuntimeError, Scope, Value};

pub fn execute_program(program: &ast::Program, scope: &mut Scope) -> Result<Value, RuntimeError> {
    evaluate_block(&program.block, scope)
}

pub fn evaluate_block(block: &ast::Block, scope: &mut Scope) -> Result<Value, RuntimeError> {
    let mut value = Value::Void;

    for statement in &block.statements {
        value = execute_statement(statement, scope)?;
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use blox_language::parse;

    use super::*;

    fn assert_result(code: &str, expected: Value) {
        let program = match parse(code) {
            Ok(program) => program,
            Err(e) => panic!("Parsing error: {}", e),
        };

        let mut scope = Scope::default();
        let result = execute_program(&program, &mut scope);

        match &result {
            Ok(value) => assert_eq!(value, &expected),
            Err(e) => panic!("Execution error: {}", e),
        }
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_let() {
        assert_result(
            "
            let x = 42
            ",
            Value::Number(42),
        );
        assert_result(
            "
            let x = 42
            x + 55
            ",
            Value::Number(97),
        );
        assert_result(
            "
            let x = 42
            let y = 55
            x + y",
            Value::Number(97),
        );
        assert_result(
            "
            let x = 50 + 5
            x
            ",
            Value::Number(55),
        );
    }

    #[test]
    fn test_function() {
        assert_result(
            "
            def add(x, y) { x + y }
            add(x: 40, y: 2)
            ",
            Value::Number(42),
        );
        assert_result(
            "
            def add(x, y) { x + y }
            let x = 40
            let y = 2
            add(x: x, y: y)
            ",
            Value::Number(42),
        );
        assert_result(
            "
            def add(x, y) { x + y }
            let x = 40
            add(x: x, y: 2)
            ",
            Value::Number(42),
        );
        assert_result(
            "
            let delta = 5
            def add_to(x) { x + delta }
            add_to(x: 50)
            ",
            Value::Number(55),
        );
    }
}
