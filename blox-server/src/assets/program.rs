use std::fmt::Display;

use blox_language::{ast, program};
use program::ProgramParser;

pub struct BloxProgram(ast::Program);

impl blox_assets::Asset for BloxProgram {
    const EXTENSIONS: &'static [&'static str] = &[".blox"];

    type Loader = BloxLoader;
}

impl Into<ast::Program> for BloxProgram {
    fn into(self) -> ast::Program {
        self.0
    }
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

impl blox_assets::Loader<BloxProgram> for BloxLoader {
    fn load(content: &[u8], _extension: &str) -> anyhow::Result<BloxProgram> {
        let input = String::from_utf8(content.to_vec())?;

        ProgramParser::new()
            .parse(&input)
            .map_err(|err| Box::new(BloxLoaderError(format!("{:?}", err))).into())
            .map(|program| BloxProgram(program))
    }
}
