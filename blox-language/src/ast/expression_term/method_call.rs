use crate::ast::Argument;

use super::{Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodCall {
    pub base: Box<Expression>,
    pub function: Identifier,
    pub arguments: Vec<Argument>,
}

impl std::fmt::Display for MethodCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}(", self.base, self.function)?;
        let arg_count = self.arguments.len();
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != arg_count - 1 {
                write!(f, "{}, ", argument)?;
            } else {
                write!(f, "{}", argument)?;
            }
        }
        write!(f, ")")
    }
}
