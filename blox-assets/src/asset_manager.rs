use std::{
    collections::{HashMap, HashSet},
    fs::DirEntry,
    path::{Component, Components, PathBuf},
    sync::{
        mpsc::{self, SendError},
        Arc,
    },
};

use notify::{Event, RecursiveMode, Watcher};
use tokio::{spawn, sync::Notify};
use tracing::{debug, error, info, info_span, instrument, warn};

use crate::{
    asset::Asset,
    error::AssetError,
    loader::Loader,
    types::{Action, AssetPath, RoutePathPart},
};

pub struct AssetManager {
    base_dir: PathBuf,
    asset_index: HashMap<AssetPath, HashSet<PathBuf>>,
    on_change: Arc<Notify>,
    pending_changes: mpsc::Receiver<PendingChange>,
}

struct PendingChange;

impl AssetManager {
    pub fn new(base_dir: &str) -> anyhow::Result<Self> {
        let base_dir = std::fs::canonicalize(base_dir)?;

        let (tx, rx) = mpsc::channel::<PendingChange>();

        let mut asset_manager = AssetManager {
            base_dir,
            asset_index: Default::default(),
            on_change: Arc::new(Notify::new()),
            pending_changes: rx,
        };

        asset_manager.reindex()?;
        asset_manager.start(tx)?;

        Ok(asset_manager)
    }

    fn start(&mut self, pending_change_sender: mpsc::Sender<PendingChange>) -> anyhow::Result<()> {
        let base_dir = self.base_dir.clone();
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(&base_dir, RecursiveMode::Recursive).unwrap();

        spawn(async move {
            info!("watching for changes in {}...", base_dir.to_string_lossy());
            for res in rx {
                match res {
                    Ok(event) => match event.kind {
                        notify::EventKind::Create(_)
                        | notify::EventKind::Modify(_)
                        | notify::EventKind::Remove(_) => {
                            info!("change detected: {:?}", event.paths);

                            if let Err(_) = pending_change_sender.send(PendingChange) {
                                break;
                            }
                        }
                        _ => {}
                    },

                    Err(e) => {
                        error!("watch error: {:?}", e);
                        break;
                    }
                }
            }
            drop(watcher);
        });

        Ok(())
    }

    pub fn on_change(&self) -> Arc<Notify> {
        self.on_change.clone()
    }

    #[instrument(skip(self))]
    pub fn process_pending_changes(&mut self) -> anyhow::Result<()> {
        let mut changed = false;

        while let Ok(_) = self.pending_changes.try_recv() {
            changed = true;
        }

        if changed {
            info!("processing changes...");
            self.reindex()?;
            self.on_change.notify_waiters();
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn load<T: Asset>(&mut self, asset_path: &AssetPath) -> anyhow::Result<T> {
        let span = info_span!("load");
        span.in_scope(|| {
            self.process_pending_changes()?;

            let file_paths = self
                .asset_index
                .get(asset_path)
                .ok_or(AssetError::AssetNotFound(asset_path.clone()))?;

            let mut matching_path = None;

            for extension in T::EXTENSIONS {
                if extension == &"*" {
                    if let Some(path) = file_paths.iter().next() {
                        matching_path = Some((path, extension));
                        break;
                    }
                } else if let Some(path) = file_paths.iter().find(|path| {
                    path.file_name()
                        .map(|f| f.to_string_lossy().ends_with(extension))
                        == Some(true)
                }) {
                    matching_path = Some((path, extension));
                    break;
                }
            }

            if let Some((path, extension)) = matching_path {
                let raw_contents = std::fs::read(path)?;
                let asset = T::Loader::load(&raw_contents, &extension)?;
                info!(path = path.to_string_lossy().to_string().as_str(), "loaded");
                Ok(asset)
            } else {
                Err(anyhow::Error::new(AssetError::NoMatchingExtension(
                    asset_path.clone(),
                    T::EXTENSIONS,
                )))
            }
        })
    }

    #[instrument(skip(self))]
    fn reindex(&mut self) -> anyhow::Result<()> {
        self.asset_index.clear();

        let files = walkdir::WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.path().canonicalize().ok());

        for path in files {
            if !path.is_file() {
                continue;
            }

            if let Ok(relative_path) = path.strip_prefix(&self.base_dir) {
                let components = relative_path.components();
                self.reindex_file(components, &path)
            }
        }

        Ok(())
    }

    fn reindex_file(&mut self, mut components: Components, path: &PathBuf) {
        if let Some(Component::Normal(c)) = components.next() {
            match c.to_str() {
                Some("app") => self.reindex_app_file(components, path),
                _ => {} // debug!("unrecognized top level file: {:?}", path)},
            }
        }
    }

    fn reindex_app_file(&mut self, mut components: Components, path: &PathBuf) {
        let Some(Component::Normal(c)) = components.next() else {
            return;
        };

        match c.to_str() {
            Some("models") => self.reindex_models_file(components, path),
            Some("routes") => self.reindex_routes_file(components, path),
            Some("static") => {
                // join together remoaining components to get the path
                let static_asset_path = components.collect::<PathBuf>();
                let static_asset_path = static_asset_path
                    .to_str()
                    .expect("path buf could not be converted to string");
                let asset_path = AssetPath::Static(static_asset_path.to_string());

                self.asset_index
                    .entry(asset_path)
                    .or_default()
                    .insert(path.clone());
            }
            _ => debug!("unrecognized app file: {:?}", path),
        }
    }

    fn reindex_models_file(&mut self, mut components: Components, path: &PathBuf) {
        let model_name = components
            .next_back()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let asset_path = AssetPath::Model(model_name.to_string());
        self.asset_index
            .entry(asset_path)
            .or_default()
            .insert(path.clone());
    }

    fn reindex_routes_file(&mut self, mut components: Components, path: &PathBuf) {
        let mut route_path_parts = Vec::new();

        if let Some(action_component) = components.next_back() {
            let action = RoutePathPart::Action(Action::from(action_component));

            for component in components {
                let part =
                    RoutePathPart::Collection(component.as_os_str().to_str().unwrap().to_string());
                route_path_parts.push(part);
            }

            route_path_parts.push(action);
        } else {
            unimplemented!();
        }

        let asset_path = AssetPath::Route(route_path_parts);
        self.asset_index
            .entry(asset_path)
            .or_default()
            .insert(path.clone());
    }

    pub fn add_root_entry(&mut self, entry: &DirEntry) -> anyhow::Result<()> {
        let filename = entry.file_name().to_string_lossy().to_string();
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
                                other => RoutePathPart::Action(Action::Custom(other.to_string())),
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
