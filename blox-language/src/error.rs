#[derive(Debug, PartialEq)]
pub enum ParseError {
    DecimalError(rust_decimal::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::DecimalError(err) => write!(f, "{}", err),
        }
    }
}

impl From<rust_decimal::Error> for ParseError {
    fn from(err: rust_decimal::Error) -> Self {
        ParseError::DecimalError(err)
    }
}
