#[derive(Debug, Clone)]
pub struct Program {
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Binding { lhs: Identifier, rhs: Expression },
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Term(ExpressionTerm),
    Operator {
        lhs: ExpressionTerm,
        operator: Operator,
        rhs: ExpressionTerm,
    },
}

#[derive(Debug, Clone)]
pub enum ExpressionTerm {
    Identifier(Identifier),
    Literal(Literal),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Add,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub ident: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub ident: Identifier,
    pub arguments: Vec<Argument>,
}
