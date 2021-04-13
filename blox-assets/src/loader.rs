use crate::Asset;

pub trait Loader<T: Asset> {
    fn load(content: &[u8], extension: &str) -> anyhow::Result<T>;
}
