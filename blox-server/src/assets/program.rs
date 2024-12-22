use std::fmt::Display;

use blox_language::{ast, parser::Parser};

pub struct BloxProgram(ast::Program);

impl blox_assets::Asset for BloxProgram {
    const EXTENSIONS: &'static [&'static str] = &[".blox"];

    type Loader = BloxLoader;
}

impl From<BloxProgram> for ast::Program {
    fn from(program: BloxProgram) -> Self {
        program.0
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
    fn load(path: &str, content: &[u8], _extension: &str) -> anyhow::Result<BloxProgram> {
        let input = String::from_utf8(content.to_vec())?;
        let parser = Parser::new(path, &input);
        parser
            .parse()
            .map_err(|err| Box::new(BloxLoaderError(format!("{:?}", err))).into())
            .map(BloxProgram)
    }
}
