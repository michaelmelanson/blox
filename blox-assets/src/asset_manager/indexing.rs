use std::path::{Component, Components, PathBuf};

use tracing::{debug, instrument};

use crate::types::{Action, AssetPath, RoutePathPart};

use super::AssetManager;

impl AssetManager {
    #[instrument(skip(self))]
    pub(super) fn reindex(&mut self) -> anyhow::Result<()> {
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
                // join together remaining components to get the path
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
}
