use crate::types::AssetPath;

#[derive(Debug)]
pub enum AssetError {
    BaseDirNotFound(String),
    AssetNotFound(AssetPath),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssetError::BaseDirNotFound(dir) => {
                write!(f, "Could not find base directory {}", dir)
            }
            AssetError::AssetNotFound(path) => {
                write!(f, "Could not find file {}", path)
            }
        }
    }
}

impl std::error::Error for AssetError {}
