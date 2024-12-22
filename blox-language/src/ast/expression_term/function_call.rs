use crate::ast::Argument;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionCall(pub Box<Expression>, pub Vec<Argument>);

impl std::fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.0)?;
        let arg_count = self.1.len();
        for (index, argument) in self.1.iter().enumerate() {
            if index != arg_count - 1 {
                write!(f, "{}, ", argument)?;
            } else {
                write!(f, "{}", argument)?;
            }
        }
        write!(f, ")")
    }
}
