use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Index,
    Show,
    New,
    Create,
    Update,
    Delete,
    Custom(String)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RoutePathPart {
    Collection(String),
    Action(Action),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AssetPath {
    Route(Vec<RoutePathPart>),
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetPath::Route(path_parts) => write!(f, "Route({:?})", path_parts),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Bindings(HashMap<String, String>);

impl Bindings {
    pub fn new(entries: &Vec<(String, String)>) -> Self {
        let mut hash = HashMap::new();

        for (key, value) in entries.clone() {
            hash.insert(key, value);
        }

        Bindings(hash)
    }
}

impl IntoIterator for Bindings {
    type Item = (String, String);

    type IntoIter = <HashMap<String, String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct Content(String);
