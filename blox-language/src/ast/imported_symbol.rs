use super::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportedSymbol(pub Identifier, pub Option<Identifier>);

impl std::fmt::Display for ImportedSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(alias) = &self.1 {
            write!(f, "{} as {}", self.0, alias)
        } else {
            write!(f, "{}", self.0)
        }
    }
}
