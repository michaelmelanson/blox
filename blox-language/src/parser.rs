use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "program.pest"]
pub struct BloxParser;
