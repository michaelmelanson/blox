use std::fmt::Display;

use crate::ast;

impl blox_assets::Asset for ast::Program {
    const EXTENSIONS: &'static [&'static str] = &[".blox"];

    type Loader = BloxLoader;
}

pub struct BloxLoader;

#[derive(Debug)]
pub struct BloxLoaderError(String);

impl Display for BloxLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("parse error: {}", self.0))
    }
}

impl std::error::Error for BloxLoaderError {}

impl blox_assets::Loader<ast::Program> for BloxLoader {
    fn load(content: &[u8], _extension: &str) -> anyhow::Result<ast::Program> {
        let input = String::from_utf8(content.to_vec())?;

        crate::program::ProgramParser::new()
            .parse(&input)
            .map_err(|err| Box::new(BloxLoaderError(format!("{:?}", err))).into())
    }
}
