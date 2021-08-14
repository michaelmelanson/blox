#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Number(i64),
    String(String),
    Symbol(String),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Number(number) => number.to_string(),
            Value::String(string) => string.clone(),
            Value::Symbol(symbol) => format!(":{}", symbol),
        }
    }
}
