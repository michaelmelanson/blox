use crate::Asset;

pub trait Loader<T: Asset> {
    fn load(path: &str, content: &[u8], extension: &str) -> anyhow::Result<T>;
}
