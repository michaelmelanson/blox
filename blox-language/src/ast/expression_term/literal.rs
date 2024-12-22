use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Boolean(bool),
    Number(Decimal),
    String(String),
    Symbol(String),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Boolean(v) => write!(f, "{v}"),
            Literal::Number(v) => write!(f, "{v}"),
            Literal::String(v) => write!(f, "'{v}'"),
            Literal::Symbol(v) => write!(f, ":{v}"),
        }
    }
}
