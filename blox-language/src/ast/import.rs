use super::ImportedSymbol;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Import(pub Vec<ImportedSymbol>, pub String);

impl std::fmt::Display for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "import {{")?;

        for (i, symbol) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", symbol)?;
        }

        write!(f, "}} from \"{}\"", self.1)
    }
}
