mod array;
mod array_index;
mod array_slice;
mod function_call;
mod if_expression;
mod literal;
mod method_call;
mod object;
mod object_index;

pub use array::Array;
pub use array_index::ArrayIndex;
pub use array_slice::ArraySlice;
pub use function_call::FunctionCall;
pub use if_expression::If;
pub use literal::Literal;
pub use method_call::MethodCall;
pub use object::Object;
pub use object_index::ObjectIndex;

use super::{Definition, Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionTerm {
    Expression(Box<Expression>),
    If(If),
    ArraySlice(ArraySlice),
    ArrayIndex(ArrayIndex),
    ObjectIndex(ObjectIndex),
    MethodCall(MethodCall),
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    Literal(Literal),
    Array(Array),
    Object(Object),
    Lambda(Definition),
}

impl std::fmt::Display for ExpressionTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionTerm::Expression(v) => write!(f, "{v}"),
            ExpressionTerm::MethodCall(v) => write!(f, "{v}"),
            ExpressionTerm::FunctionCall(v) => write!(f, "{v}"),
            ExpressionTerm::Identifier(v) => write!(f, "{v}"),
            ExpressionTerm::Literal(v) => write!(f, "{v}"),
            ExpressionTerm::Array(v) => write!(f, "{v}"),
            ExpressionTerm::ArraySlice(v) => write!(f, "{v}"),
            ExpressionTerm::ArrayIndex(v) => write!(f, "{v}"),
            ExpressionTerm::Object(v) => write!(f, "{v}"),
            ExpressionTerm::ObjectIndex(v) => write!(f, "{v}"),
            ExpressionTerm::If(v) => write!(f, "{v}"),
            ExpressionTerm::Lambda(v) => write!(f, "{v}"),
        }
    }
}
