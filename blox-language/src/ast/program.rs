use crate::location::Location;

use super::Block;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub block: Block,
    pub location: Location,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.block)
    }
}
