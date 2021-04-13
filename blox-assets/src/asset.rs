use crate::Loader;

pub trait Asset: Sized {
    const EXTENSIONS: &'static [&'static str];
    type Loader: Loader<Self>;
}
