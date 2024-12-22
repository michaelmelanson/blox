#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,
    pub range: tree_sitter::Range,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file, self.range.start_point.row, self.range.start_point.column
        )
    }
}
