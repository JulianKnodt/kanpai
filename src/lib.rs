#![feature(let_else)]

#[allow(unused)]
pub mod ast;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub kanpai);

pub use kanpai::ProgramParser;
