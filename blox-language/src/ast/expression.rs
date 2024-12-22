use super::{ExpressionTerm, Operator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Term(ExpressionTerm),
    BinaryExpression(Box<Expression>, Operator, Box<Expression>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Term(v) => write!(f, "{}", v),
            Expression::BinaryExpression(lhs, operator, rhs) => {
                write!(f, "({} {} {})", lhs, operator, rhs)
            }
        }
    }
}
