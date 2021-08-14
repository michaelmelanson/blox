#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Binding { lhs: Identifier, rhs: Expression },
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Term(ExpressionTerm),
    Operator {
        lhs: ExpressionTerm,
        operator: Operator,
        rhs: ExpressionTerm,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionTerm {
    Identifier(Identifier),
    Literal(Literal),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(i64),
    String(String),
    Symbol(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Multiply,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub identifier: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub identifier: Identifier,
    pub arguments: Vec<Argument>,
}
