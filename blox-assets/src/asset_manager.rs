use std::{
    collections::{HashMap, HashSet},
    fs::DirEntry,
    path::PathBuf,
};

use tracing::{debug, error, info, info_span, instrument, warn};

use crate::{
    asset::Asset,
    error::AssetError,
    loader::Loader,
    types::{AssetPath, Action, RoutePathPart},
};

#[derive(Clone)]
pub struct AssetManager {
    base_dir: PathBuf,
    asset_index: HashMap<AssetPath, HashSet<PathBuf>>,
}

impl AssetManager {
    pub fn new(base_dir: &str) -> anyhow::Result<Self> {
        let base_dir = std::fs::canonicalize(base_dir)?;

        let mut asset_manager = AssetManager {
            base_dir,
            asset_index: Default::default(),
        };

        asset_manager.start()?;

        Ok(asset_manager)
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.reindex()?;

        info!("after initial index: {:#?}", self.asset_index);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn load<T: Asset>(&mut self, asset_path: &AssetPath) -> anyhow::Result<T> {
        let span = info_span!("load");
        span.in_scope(|| {
            let file_paths = self
                .asset_index
                .get(asset_path)
                .ok_or(AssetError::AssetNotFound(asset_path.clone()))?;

            let mut matching_path = None;

            for extension in T::EXTENSIONS {
                if let Some(path) = file_paths.iter().find(|path| path.file_name().map(|f| f.to_string_lossy().ends_with(extension)) == Some(true)) {
                    matching_path = Some((path, extension));
                    break;
                }
            }

            if let Some((path, extension)) = matching_path {
                debug!(path = path.to_string_lossy().to_string().as_str(), "trying");

                let raw_contents = std::fs::read(path)?;
                let asset = T::Loader::load(&raw_contents, &extension)?;
                info!(path = path.to_string_lossy().to_string().as_str(), "loaded");
                Ok(asset)
            } else {
                Err(anyhow::Error::new(AssetError::NoMatchingExtension(
                    asset_path.clone(),
                    T::EXTENSIONS
                )))
            }
        })
    }

    #[instrument(skip(self))]
    fn reindex(&mut self) -> anyhow::Result<()> {
        self.asset_index.clear();

        for entry in std::fs::read_dir(&self.base_dir)? {
            match entry {
                Ok(entry) => self.add_root_entry(&entry)?,

                Err(error) => {
                    error!(
                        error = error.to_string().as_str(),
                        "error reading directory entry"
                    );
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, entry), fields(name))]
    pub fn add_root_entry(&mut self, entry: &DirEntry) -> anyhow::Result<()> {
        let filename = entry.file_name().to_string_lossy().to_string();
        tracing::Span::current().record("name", &filename.as_str());

        match filename.as_str() {
            "routes" => {
                self.add_route_directory(&entry, vec![])?;
            }
            _ => {
                warn!("don't know what this is");
            }
        }

        Ok(())
    }

    #[instrument(skip(self, entry), fields(name))]
    pub fn add_route_directory(
        &mut self,
        entry: &DirEntry,
        path: Vec<RoutePathPart>,
    ) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(entry.path())? {
            match entry {
                Ok(entry) => {
                    let name = entry.file_name().to_string_lossy().to_string();

                    if entry.file_type()?.is_dir() {
                        let mut new_path = path.clone();
                        new_path.push(RoutePathPart::Collection(name));

                        self.add_route_directory(&entry, new_path)?;
                    } else {
                        info!(name = name.as_str(), "file");

                        if let Some(base_name) = name.split('.').next() {
                            let part = match base_name {
                                "index" => RoutePathPart::Action(Action::Index),
                                "show" => RoutePathPart::Action(Action::Show),
                                other => {
                                    RoutePathPart::Action(Action::Custom(other.to_string()))
                                }
                            };

                            let mut file_asset_path = path.clone();
                            file_asset_path.push(part);

                            self.asset_index
                                .entry(AssetPath::Route(file_asset_path))
                                .or_default()
                                .insert(entry.path());
                        }
                    }
                }

                Err(error) => error!(error = error.to_string().as_str()),
            }
        }

        Ok(())
    }
}
