use std::sync::Arc;

use blox_language::ast;

use crate::{statement::execute_statement, RuntimeError, Scope, Value};

pub fn execute_program(
    program: &ast::Program,
    scope: &mut Arc<Scope>,
) -> Result<Value, RuntimeError> {
    evaluate_block(&program.0, scope)
}

pub fn evaluate_block(block: &ast::Block, scope: &mut Arc<Scope>) -> Result<Value, RuntimeError> {
    let mut value = Value::Void;

    for statement in &block.0 {
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

        let mut scope = Arc::new(Scope::default());
        let result = execute_program(&program, &mut scope);

        match &result {
            Ok(value) => assert_eq!(
                value, &expected,
                "Expected: {expected}, got: {value}\nInput: {code}\nAST: {program:?}"
            ),
            Err(e) => panic!("Execution error: {}", e),
        }
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_boolean() {
        assert_result("true", Value::Boolean(true));
        assert_result("false", Value::Boolean(false));
    }

    #[test]
    fn test_expressions() {
        assert_result(
            "
            1 + 2 + 3
            ",
            Value::Number(6.into()),
        );
        assert_result(
            "
            1 + 2 * 3
            ",
            Value::Number(7.into()),
        );
        assert_result(
            "
            (1 + 2) * 3
            ",
            Value::Number(9.into()),
        );
        assert_result(
            "
            1 + 2 * 3 + 4
            ",
            Value::Number(11.into()),
        );
    }

    #[test]
    fn test_let_numbers() {
        assert_result(
            "
            let x = 42
            ",
            Value::Number(42.into()),
        );
        assert_result(
            "
            let x = 42
            x + 55
            ",
            Value::Number(97.into()),
        );
        assert_result(
            "
            let x = 42
            let y = 55
            x + y",
            Value::Number(97.into()),
        );
        assert_result(
            "
            let x = 50 + 5
            x
            ",
            Value::Number(55.into()),
        );
        assert_result(
            "
            let x = 50
            let y = 5
            let z = 123
            x + y + z
            ",
            Value::Number(178.into()),
        )
    }

    #[test]
    fn test_let_strings() {
        assert_result(
            "
            let x = \"hello\"
            ",
            Value::String("hello".to_string()),
        );
        assert_result(
            "
            let x = 'hello'
            ",
            Value::String("hello".to_string()),
        );
        assert_result(
            "
            'hello' ++ ' world'
            ",
            Value::String("hello world".to_string()),
        );
        assert_result(
            "
            let x = 'hello'
            x ++ ' world'
            ",
            Value::String("hello world".to_string()),
        );
    }

    #[test]
    fn test_let_arrays() {
        assert_result(
            "
            let x = [1, 2, 3]
            ",
            Value::Array(vec![
                Value::Number(1.into()),
                Value::Number(2.into()),
                Value::Number(3.into()),
            ]),
        );
        assert_result(
            "
            let x = [1 + 1, 2 + 2, 3 + 3]
            ",
            Value::Array(vec![
                Value::Number(2.into()),
                Value::Number(4.into()),
                Value::Number(6.into()),
            ]),
        );
        assert_result(
            "
            let x = [1, 2, 3]
            x[1]
            ",
            Value::Number(2.into()),
        );
        assert_result(
            "
            let x = [1, 2, 3]
            x[0] + x[2]
            ",
            Value::Number(4.into()),
        );
        assert_result(
            "
            def numbers() { [1, 2, 3] }
            numbers()[2]
            ",
            Value::Number(3.into()),
        );
        assert_result(
            "
            [4, 5, 6][1]
            ",
            Value::Number(5.into()),
        );
    }

    #[test]
    fn test_objects() {
        assert_result(
            "
            let x = { a: 1, b: 2 }
            ",
            Value::Object(
                [
                    ("a".to_string(), Value::Number(1.into())),
                    ("b".to_string(), Value::Number(2.into())),
                ]
                .into(),
            ),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a
            ",
            Value::Number(1.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.b
            ",
            Value::Number(2.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b
            ",
            Value::Number(3.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + 3
            ",
            Value::Number(4.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b + 3
            ",
            Value::Number(6.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b + x.a
            ",
            Value::Number(4.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b + x.b
            ",
            Value::Number(5.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b + x.a + x.b
            ",
            Value::Number(6.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b + x.a + x.b + 3
            ",
            Value::Number(9.into()),
        );
        assert_result(
            "
            let x = { a: 1, b: 2 }
            x.a + x.b + x.a + x.b + x.a
            ",
            Value::Number(7.into()),
        );
    }

    #[test]
    fn test_function() {
        assert_result(
            "
            def add(x, y) { x + y }
            add(x: 40, y: 2)
            ",
            Value::Number(42.into()),
        );
        assert_result(
            "
            def add(x, y) { x + y }
            let x = 40
            let y = 2
            add(x: x, y: y)
            ",
            Value::Number(42.into()),
        );
        assert_result(
            "
            def add(x, y) { x + y }
            let x = 40
            add(x: x, y: 2)
            ",
            Value::Number(42.into()),
        );
        assert_result(
            "
            let delta = 5
            def add_to(x) { x + delta }
            add_to(x: 50)
            ",
            Value::Number(55.into()),
        );
    }

    #[test]
    fn test_if() {
        assert_result(
            "
            if 0 { 'then' } else { 'else' }
            ",
            Value::String("else".to_string()),
        );
        assert_result(
            "
            if 0 { 'then' }
            ",
            Value::Void,
        );
        assert_result(
            "
            if 1 { 'then' } else { 'else' }
            ",
            Value::String("then".to_string()),
        );
        assert_result(
            "
            if -1 { 'then' } else { 'else' }
            ",
            Value::String("else".to_string()),
        );
        assert_result(
            "
            let x = 0
            if x { 'then' } else { 'else' }
            ",
            Value::String("else".to_string()),
        );
        assert_result(
            "
            let x = 1
            if x { 'then' } else { 'else' }
            ",
            Value::String("then".to_string()),
        );

        assert_result("if 1 == 1 { 'ok' }", Value::String("ok".to_string()));
        assert_result("if 1 == 2 { 'bork' }", Value::Void);
        assert_result("if true == 1 { 'bork' }", Value::Void);
        assert_result("if false == 1 { 'bork' }", Value::Void);
        assert_result("if 1 == true { 'bork' }", Value::Void);
        assert_result("if 1 == false { 'bork' }", Value::Void);
        assert_result("if true == true { 'ok' }", Value::String("ok".to_string()));
        assert_result(
            "if false == false { 'ok' }",
            Value::String("ok".to_string()),
        );
        assert_result(
            "if 'blox' == 'blox' { 'ok' }",
            Value::String("ok".to_string()),
        );
        assert_result("if 'blox' == 'ruby' { 'bork' }", Value::Void);

        assert_result("if 1 < 2 { 'ok' }", Value::String("ok".to_string()));
        assert_result("if 2 < 2 { 'error' }", Value::Void);
        assert_result("if 2 < 2 { 'error' }", Value::Void);
    }

    #[test]
    pub fn test_fib() {
        assert_result(
            "
            def fib(x) {
              if x == 0 {
                0
              } else if x == 1 {
                1
              } else {
                fib(x: x - 2) + fib(x: x - 1)
              }
            }
            fib(x: 10)
            ",
            Value::Number(55.into()),
        );
    }
}
