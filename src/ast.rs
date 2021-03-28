#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Endpoint(EndpointDeclaration),
}

#[derive(Debug, Clone)]
pub struct EndpointDeclaration {
    pub verb: HttpVerb,
    pub path: HttpPath,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub enum HttpVerb {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone)]
pub struct HttpPath { 
  pub parts: Vec<HttpPathPart> 
}

#[derive(Debug, Clone)]
pub enum HttpPathPart {
    Literal(String),
    Variable(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub struct Block {
  pub statements: Vec<BlockStatement>
}

#[derive(Debug, Clone)]
pub enum BlockStatement {
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
    String(String)
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
