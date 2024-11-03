#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Void,
    Number(i64),
    String(String),
    Symbol(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Value::Void => "(void)".to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => format!("'{string}'"),
            Value::Symbol(symbol) => format!(":{}", symbol),
        };

        write!(f, "{}", s)
    }
}
