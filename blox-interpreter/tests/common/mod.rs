use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use blox_interpreter::{execute_program, load_stdlib, EvaluationContext, Scope, Value};
use blox_language::{ast, error::ParseError, parser::Parser};

pub fn parse(code: &str) -> Result<ast::Program, ParseError> {
    let parser = Parser::new(code);
    parser.parse()
}

pub fn assert_result(code: &str, expected: Value) {
    let program = match parse(code) {
        Ok(program) => program,
        Err(e) => panic!("Parsing error: {}", e),
    };

    let mut context = EvaluationContext::new(
        "..",
        Arc::new(Scope::default()),
        Arc::new(RwLock::new(BTreeMap::new())),
    );
    load_stdlib(&mut context);

    let result = execute_program(&program, &mut context);

    match &result {
        Ok(value) => assert_eq!(
            value, &expected,
            "Expected: {expected}, got: {value}\nInput: {code}\nAST: {program:?}"
        ),
        Err(e) => panic!("Execution error: {}\n{program}", e),
    }
    assert_eq!(result, Ok(expected));
}
