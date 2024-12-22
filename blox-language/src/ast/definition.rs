use super::{Block, Identifier, Parameter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub name: Option<Identifier>,
    pub parameters: Vec<Parameter>,
    pub body: Block,
}

impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "def {}(", name)?;
        } else {
            write!(f, "|")?;
        }

        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", param)?;
        }

        if self.name.is_none() {
            write!(f, "| {}", self.body)
        } else {
            write!(f, ") {}", self.body)
        }
    }
}
