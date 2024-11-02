use hyper::Response;

#[derive(Clone)]
pub struct StaticFile(Vec<u8>);

impl Into<Response<http_body_util::Full<hyper::body::Bytes>>> for StaticFile {
    fn into(self) -> Response<http_body_util::Full<hyper::body::Bytes>> {
        Response::new(self.0.into())
    }
}

impl blox_assets::Asset for StaticFile {
    const EXTENSIONS: &'static [&'static str] = &["*"];
    type Loader = StaticFileLoader;
}

pub struct StaticFileLoader;

impl blox_assets::Loader<StaticFile> for StaticFileLoader {
    fn load(content: &[u8], _filename: &str) -> Result<StaticFile, anyhow::Error> {
        Ok(StaticFile(content.to_vec()))
    }
}

#[derive(Debug)]
pub struct StaticFileError;

impl std::fmt::Display for StaticFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("static file errror"))
    }
}

impl std::error::Error for StaticFileError {}
