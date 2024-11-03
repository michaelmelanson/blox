#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(pub String);

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for statement in &self.statements {
            writeln!(f, "{}", statement)?;
        }

        write!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Definition(Definition),
    Expression(Expression),
    Binding { lhs: Identifier, rhs: Expression },
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Definition(def) => write!(f, "{}", def),
            Statement::Expression(expr) => write!(f, "{}", expr),
            Statement::Binding { lhs, rhs } => write!(f, "let {} = {}", lhs.0, rhs),
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
pub struct Parameter(pub Identifier);

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Term(ExpressionTerm),
    Operator {
        lhs: ExpressionTerm,
        operator: Operator,
        rhs: ExpressionTerm,
    },
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Term(v) => write!(f, "{}", v),
            Expression::Operator { lhs, operator, rhs } => {
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
    Expression(Box<Expression>),
}

impl std::fmt::Display for ExpressionTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionTerm::FunctionCall(v) => write!(f, "{}", v),
            ExpressionTerm::Identifier(v) => write!(f, "{}", v),
            ExpressionTerm::Literal(v) => write!(f, "{}", v),
            ExpressionTerm::Expression(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Number(i64),
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
pub enum Operator {
    Add,
    Multiply,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Multiply => write!(f, "+"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Argument {
    pub identifier: Identifier,
    pub value: Expression,
}

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.identifier, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionCall {
    pub identifier: Identifier,
    pub arguments: Vec<Argument>,
}

impl std::fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.identifier)?;
        let arg_count = self.arguments.len();
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != arg_count - 1 {
                write!(f, "{}, ", argument)?;
            } else {
                write!(f, "{}", argument)?;
            }
        }
        write!(f, ")")
    }
}
