use blox_assets::types::{Action, AssetPath, Bindings, RoutePathPart};
use hyper::{Method, Uri};
use tracing::debug;

#[derive(Debug, PartialEq)]
pub enum RoutingError {}

impl std::fmt::Display for RoutingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "routing error")
    }
}

impl std::error::Error for RoutingError {}

fn singularize(s: &str) -> String {
    s.trim_end_matches('s').to_string()
}

fn id_key_from_collection_name(collection: &str) -> String {
    let singular = singularize(collection);
    format!("{}_id", singular)
}

pub fn request_asset_path(
    method: &Method,
    uri: &Uri,
) -> Result<(AssetPath, Bindings), RoutingError> {
    let mut route_parts = Vec::new();
    let mut bindings = Vec::new();

    let path = uri.path().trim_matches('/');
    if path.is_empty() {
        route_parts.push(RoutePathPart::Action(Action::Index));
    } else {
        let mut parts = path.split("/").collect::<Vec<_>>().into_iter().peekable();

        if parts.peek() == Some(&"static") {
            let static_path = parts.skip(1).collect::<Vec<_>>().join("/").to_string();
            debug!(static_path, path, "static");

            return Ok((AssetPath::Static(static_path), Bindings::default()));
        }

        while let Some(collection) = parts.next() {
            route_parts.push(RoutePathPart::Collection(collection.to_string()));

            let id_key = id_key_from_collection_name(collection);

            match (method, parts.next()) {
                (&Method::GET, None) => {
                    route_parts.push(RoutePathPart::Action(Action::Index));
                }

                (&Method::GET, Some("new")) => {
                    route_parts.push(RoutePathPart::Action(Action::New));
                }

                (&Method::GET, Some(id)) => {
                    route_parts.push(RoutePathPart::Action(Action::Show));
                    bindings.push((id_key, id.to_string()));
                }

                (&Method::PUT, Some(id)) => {
                    route_parts.push(RoutePathPart::Action(Action::Update));
                    bindings.push((id_key, id.to_string()));
                }

                (&Method::DELETE, Some(id)) => {
                    route_parts.push(RoutePathPart::Action(Action::Delete));
                    bindings.push((id_key, id.to_string()));
                }

                (&Method::POST, None) => {
                    route_parts.push(RoutePathPart::Action(Action::Create));
                }

                (method, part) => unimplemented!("method={} part={:?}", method, part),
            }
        }
    }

    Ok((AssetPath::Route(route_parts), Bindings::new(&bindings)))
}

#[cfg(test)]
mod test {
    use blox_assets::types::{Action, AssetPath, Bindings, RoutePathPart};
    use hyper::{Method, Uri};

    use crate::request_asset_path;

    fn uri_with_path_and_query(path_and_query: &str) -> Uri {
        Uri::builder()
            .path_and_query(path_and_query)
            .build()
            .unwrap()
    }

    #[test]
    fn test_request_asset_path() {
        assert_eq!(
            request_asset_path(&Method::GET, &uri_with_path_and_query("/")),
            Ok((
                AssetPath::Route(vec![RoutePathPart::Action(Action::Index)]),
                Bindings::default()
            ))
        );

        assert_eq!(
            request_asset_path(&Method::GET, &uri_with_path_and_query("/lists")),
            Ok((
                AssetPath::Route(vec![
                    RoutePathPart::Collection("lists".to_string()),
                    RoutePathPart::Action(Action::Index)
                ]),
                Bindings::default()
            ))
        );

        assert_eq!(
            request_asset_path(&Method::GET, &uri_with_path_and_query("/lists/1")),
            Ok((
                AssetPath::Route(vec![
                    RoutePathPart::Collection("lists".to_string()),
                    RoutePathPart::Action(Action::Show)
                ]),
                Bindings::new(&vec![("list_id".to_string(), "1".to_string())])
            ))
        );

        assert_eq!(
            request_asset_path(&Method::GET, &uri_with_path_and_query("/lists/new")),
            Ok((
                AssetPath::Route(vec![
                    RoutePathPart::Collection("lists".to_string()),
                    RoutePathPart::Action(Action::New)
                ]),
                Bindings::default()
            ))
        );

        assert_eq!(
            request_asset_path(&Method::POST, &uri_with_path_and_query("/lists")),
            Ok((
                AssetPath::Route(vec![
                    RoutePathPart::Collection("lists".to_string()),
                    RoutePathPart::Action(Action::Create)
                ]),
                Bindings::default()
            ))
        );

        assert_eq!(
            request_asset_path(&Method::DELETE, &uri_with_path_and_query("/lists/1")),
            Ok((
                AssetPath::Route(vec![
                    RoutePathPart::Collection("lists".to_string()),
                    RoutePathPart::Action(Action::Delete)
                ]),
                Bindings::new(&vec![("list_id".to_string(), "1".to_string())])
            ))
        );

        assert_eq!(
            request_asset_path(&Method::PUT, &uri_with_path_and_query("/lists/1")),
            Ok((
                AssetPath::Route(vec![
                    RoutePathPart::Collection("lists".to_string()),
                    RoutePathPart::Action(Action::Update)
                ]),
                Bindings::new(&vec![("list_id".to_string(), "1".to_string())])
            ))
        );
    }
}
