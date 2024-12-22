use crate::ast::{Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectIndex {
    pub base: Box<Expression>,
    pub index: Identifier,
}

impl std::fmt::Display for ObjectIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.base, self.index)
    }
}
