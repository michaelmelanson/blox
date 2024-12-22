use super::{Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Argument(pub Identifier, pub Expression);

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}
