pub mod ast;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub program);
