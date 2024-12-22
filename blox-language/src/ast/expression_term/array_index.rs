use crate::ast::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayIndex {
    pub base: Box<Expression>,
    pub index: Box<Expression>,
}

impl std::fmt::Display for ArrayIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.base, self.index)
    }
}
