use crate::location::Location;

use super::Statement;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub location: Location,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;

        for (index, statement) in self.statements.iter().enumerate() {
            write!(f, "{}", statement)?;

            if index < self.statements.len() - 1 {
                writeln!(f, ";")?;
            }
        }

        write!(f, " }}")
    }
}
