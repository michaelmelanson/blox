
use blox_assets::types::AssetPath;


#[derive(Debug)]
pub enum RoutingError {

}

impl std::fmt::Display for RoutingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "routing error")
    }
}

impl std::error::Error for RoutingError {}

pub fn request_asset_path<T>(request: &hyper::Request<T>) -> Result<AssetPath, RoutingError> {
  let uri = request.uri();
  let method = request.method();
  
  match method {
    &hyper::Method::GET => {
      Ok(AssetPath::new(
         std::path::Path::new("routes")
            .join(uri.path().trim_matches('/'))
            .join("index")
            .to_string_lossy()
            .to_string()
      ))
    },
    method => unimplemented!("method {}", method)
  }
}