use crate::ast::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArraySlice {
    pub base: Box<Expression>,
    pub start: Option<Box<Expression>>,
    pub end: Option<Box<Expression>>,
}

impl std::fmt::Display for ArraySlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[", self.base)?;
        if let Some(start) = &self.start {
            write!(f, "{start}")?;
        }
        write!(f, "..")?;
        if let Some(end) = &self.end {
            write!(f, "{end}")?;
        }
        write!(f, "]")
    }
}
