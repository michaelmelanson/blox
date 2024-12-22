use crate::ast::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array(pub Vec<Expression>);

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, member) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", member)?;
        }
        write!(f, "]")
    }
}
