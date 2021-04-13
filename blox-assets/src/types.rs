use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct AssetPath(String);

impl AssetPath {
    pub fn new(path: String) -> Self {
        AssetPath(path)
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Content(String);
