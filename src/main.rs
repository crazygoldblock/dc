//#![no_std]

use std::env::args;

use crate::compiler::compile;


mod lexer;
mod parser;
mod arena;
mod ir_lowerer;
mod test;
mod exe_builder;
mod assembly_lowerer;
mod compiler;
fn main() {
    let mut args = args();
    let input = args.nth(1).unwrap();
    let output = match args.next() {
        Some(p) => p,
        None => "testexe2.exe".to_string(),
    };

    compile(&input, &output);
}
