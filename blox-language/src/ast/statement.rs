use super::{Definition, Expression, Identifier, Import};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Definition(Definition),
    Binding(Identifier, Expression),
    Import(Import),
    Expression(Expression),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Definition(def) => write!(f, "{}", def),
            Statement::Binding(lhs, rhs) => write!(f, "let {} = {}", lhs.name, rhs),
            Statement::Import(import) => write!(f, "{}", import),
            Statement::Expression(expr) => write!(f, "{}", expr),
        }
    }
}
