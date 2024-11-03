mod error;
mod expression;
mod program;
mod scope;
mod statement;
mod value;

pub use self::{error::RuntimeError, program::execute_program, scope::Scope, value::Value};
