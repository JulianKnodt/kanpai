#![feature(let_else, box_patterns, box_syntax)]

#[allow(unused)]
pub mod ast;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub kanpai);

pub use self::kanpai::ProgramParser;
