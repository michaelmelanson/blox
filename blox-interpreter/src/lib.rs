use std::{collections::HashMap, convert::Infallible};

use blox_language::ast;

pub fn execute_program(program: &ast::Program, scope: &mut Scope) -> Result<(), Infallible> {
  for statement in &program.block.statements {
      match statement {
          ast::Statement::Binding { lhs, rhs } => {
              if let Some(value) = evaluate_expression(rhs, &scope) {
                  scope.bindings.insert(lhs.clone(), value);
              } else {
                  unimplemented!();
              }
          }

          ast::Statement::FunctionCall(_call) => {
              unimplemented!()
          }
      }
  }

  Ok::<_, Infallible>(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
  Number(i64),
  String(String),
}

impl ToString for Value {
  fn to_string(&self) -> String {
      match self {
          Value::Number(number) => number.to_string(),
          Value::String(string) => string.clone(),
      }
  }
}

pub fn evaluate_expression(expression: &ast::Expression, scope: &Scope) -> Option<Value> {
  match expression {
      ast::Expression::Term(term) => evaluate_expression_term(term, scope),
      ast::Expression::Operator { lhs, operator, rhs } => {
          let lhs_value = evaluate_expression_term(lhs, scope);
          let rhs_value = evaluate_expression_term(rhs, scope);

          match operator {
              ast::Operator::Add => match (lhs_value, rhs_value) {
                  (Some(Value::String(lhs)), Some(Value::String(rhs))) => {
                      Some(Value::String(lhs + &rhs))
                  }
                  _ => None,
              },
          }
      }
  }
}

pub fn evaluate_expression_term(term: &ast::ExpressionTerm, scope: &Scope) -> Option<Value> {
  match term {
      ast::ExpressionTerm::Identifier(identifier) => {
          scope.bindings.get(identifier).clone().map(|x| x.clone())
      }
      ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Some(Value::Number(*number)),
      ast::ExpressionTerm::Literal(ast::Literal::String(string)) => {
          Some(Value::String(string.clone()))
      }
      ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, scope),
  }
}


#[derive(Default, Debug)]
pub struct Scope {
    pub bindings: HashMap<ast::Identifier, Value>,
}

impl Scope {
    pub fn child(&self) -> Self {
        Scope {
            bindings: self.bindings.clone(),
        }
    }
}
