use std::sync::Arc;

use blox_interpreter::{execute_program, Scope, Value};
use blox_language::{ast, ParseError, Parser};

pub fn parse(code: &str) -> Result<ast::Program, ParseError> {
    let parser = Parser::new(code);
    parser.parse()
}

pub fn assert_result(code: &str, expected: Value) {
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
        Err(e) => panic!("Execution error: {}\n{program}", e),
    }
    assert_eq!(result, Ok(expected));
}
