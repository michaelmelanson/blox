use crate::ast::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object(pub Vec<(String, Expression)>);

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (i, (key, value)) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}: {}", key, value)?;
        }

        write!(f, "}}")
    }
}
