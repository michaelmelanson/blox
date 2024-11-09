use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Block);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(pub String);

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block(pub Vec<Statement>);

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for statement in &self.0 {
            writeln!(f, "{}", statement)?;
        }

        write!(f, "}}")
    }
}

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
            Statement::Binding(lhs, rhs) => write!(f, "let {} = {}", lhs.0, rhs),
            Statement::Import(import) => write!(f, "{}", import),
            Statement::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub body: Block,
}

impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "def {}(", self.name)?;

        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", param)?;
        }

        write!(f, ") {}", self.body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Import(pub Vec<ImportedSymbol>, pub String);

impl std::fmt::Display for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "import {{")?;

        for (i, symbol) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", symbol)?;
        }

        write!(f, "}} from \"{}\"", self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportedSymbol(pub Identifier, pub Option<Identifier>);

impl std::fmt::Display for ImportedSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(alias) = &self.1 {
            write!(f, "{} as {}", self.0, alias)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter(pub Identifier);

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Term(ExpressionTerm),
    Operator(Box<Expression>, Operator, Box<Expression>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Term(v) => write!(f, "{}", v),
            Expression::Operator(lhs, operator, rhs) => {
                write!(f, "({} {} {})", lhs, operator, rhs)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionTerm {
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    Literal(Literal),
    Array(Array),
    ArrayIndex(ArrayIndex),
    Object(Object),
    ObjectIndex(ObjectIndex),
    Expression(Box<Expression>),
    If(If),
}

impl std::fmt::Display for ExpressionTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionTerm::FunctionCall(v) => write!(f, "{v}"),
            ExpressionTerm::Identifier(v) => write!(f, "{v}"),
            ExpressionTerm::Literal(v) => write!(f, "{v}"),
            ExpressionTerm::Expression(v) => write!(f, "{v}"),
            ExpressionTerm::Array(v) => write!(f, "{v}"),
            ExpressionTerm::ArrayIndex(v) => write!(f, "{v}"),
            ExpressionTerm::Object(v) => write!(f, "{v}"),
            ExpressionTerm::ObjectIndex(v) => write!(f, "{v}"),
            ExpressionTerm::If(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayIndex {
    pub array: Box<ExpressionTerm>,
    pub index: Box<Expression>,
}

impl std::fmt::Display for ArrayIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.array, self.index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object(pub Vec<(String, Expression)>);

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (i, (key, value)) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}: {}", key, value)?;
        }

        write!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectIndex {
    pub object: Box<ExpressionTerm>,
    pub key: String,
}

impl std::fmt::Display for ObjectIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.object, self.key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Number(Decimal),
    String(String),
    Symbol(String),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(v) => write!(f, "{}", v),
            Literal::String(v) => write!(f, "'{}'", v),
            Literal::Symbol(v) => write!(f, ":{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array(pub Vec<Expression>);

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, member) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", member)?;
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    Add,
    Multiply,
    Concatenate,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Multiply => write!(f, "+"),
            Operator::Concatenate => write!(f, "++"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Argument(pub Identifier, pub Expression);

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.0, self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionCall(pub Identifier, pub Vec<Argument>);

impl std::fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.0)?;
        let arg_count = self.1.len();
        for (index, argument) in self.1.iter().enumerate() {
            if index != arg_count - 1 {
                write!(f, "{}, ", argument)?;
            } else {
                write!(f, "{}", argument)?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct If {
    pub condition: Box<Expression>,
    pub then_branch: Box<Block>,
    pub else_branch: Option<Box<Block>>,
}

impl std::fmt::Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(else_branch) = &self.else_branch {
            write!(
                f,
                "if {} then {} else {}",
                self.condition, self.then_branch, else_branch
            )
        } else {
            write!(f, "if {} then {}", self.condition, self.then_branch)
        }
    }
}
