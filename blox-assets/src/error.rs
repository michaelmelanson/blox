use crate::types::AssetPath;

#[derive(Debug)]
pub enum AssetError {
    BaseDirNotFound(String),
    AssetNotFound(AssetPath),
    NoMatchingExtension(AssetPath, &'static [&'static str]),
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
            AssetError::NoMatchingExtension(path, extensions) => {
                write!(
                    f,
                    "No assets for {} match extensions {:?}",
                    path, extensions
                )
            }
        }
    }
}

impl std::error::Error for AssetError {}
