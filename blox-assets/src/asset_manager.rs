use std::path::PathBuf;

use tracing::{error, info_span, info, debug};

use crate::{
    asset::Asset,
    error::AssetError,
    loader::Loader,
    types::{AssetPath},
};

#[derive(Clone)]
pub struct AssetManager {
    base_dir: PathBuf
}

impl AssetManager {
    pub fn new(base_dir: &str) -> anyhow::Result<Self> {
        let base_dir = std::fs::canonicalize(base_dir)?;
        Ok(AssetManager {
            base_dir: base_dir
        })
    }

    pub fn load<T: Asset>(&mut self, asset_path: &AssetPath) -> anyhow::Result<T> {
        let span = info_span!("load");
        span.in_scope(|| {
            for extension in T::EXTENSIONS {
                let path =
                    self.base_dir
                        .join(format!("{}{}", asset_path, extension));

                debug!(path=path.to_string_lossy().to_string().as_str(), "trying");

                if let Ok(canonical_path) = std::fs::canonicalize(&path) {
                    if canonical_path.starts_with(&self.base_dir) {
                        let raw_contents = std::fs::read(canonical_path)?;
                        let asset = T::Loader::load(&raw_contents, &extension)?;
                        info!(
                            path=path.to_string_lossy().to_string().as_str(), 
                            "loaded"
                        );
                        return Ok(asset);
                    } else {
                        error!(
                            path=path.to_str().unwrap(),
                            "not inside base directory"
                        );
                    }
                } else {
                    let path = path.to_string_lossy().to_string();
                    error!(
                        path=path.as_str(),
                        "could not canonicalize path"
                    );
                }
            }

            Err(anyhow::Error::new(AssetError::AssetNotFound(asset_path.clone())))
        })
    }
}
