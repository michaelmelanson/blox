#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    // unary
    Negate,
    Not,

    // binary
    Add,
    Subtract,
    Multiply,
    Divide,
    Concatenate,
    Equal,
    NotEqual,
    GreaterOrEqual,
    GreaterThan,
    LessOrEqual,
    LessThan,

    Assignment,
    Append,
    Pipe,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Negate => write!(f, "-"),
            Operator::Not => write!(f, "!"),
            Operator::Add => write!(f, "+"),
            Operator::Subtract => write!(f, "-"),
            Operator::Multiply => write!(f, "+"),
            Operator::Divide => write!(f, "/"),
            Operator::Concatenate => write!(f, "++"),
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
            Operator::GreaterOrEqual => write!(f, ">="),
            Operator::GreaterThan => write!(f, ">"),
            Operator::LessOrEqual => write!(f, "<="),
            Operator::LessThan => write!(f, "<"),
            Operator::Assignment => write!(f, "="),
            Operator::Append => write!(f, "<<"),
            Operator::Pipe => write!(f, "|>"),
        }
    }
}
