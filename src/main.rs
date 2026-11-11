//#![no_std]

use std::{env::args, fs::write};

use crate::compiler::compile;

mod lexer;
mod parser;
mod arena;
mod ir_lowerer;
mod exe_builder;
mod assembly_lowerer;
mod compiler;
mod test;

fn main() {
    let mut args = args();
    let input = args.nth(1).unwrap();
    let output = match args.next() {
        Some(p) => p,
        None => "output_executable.exe".to_string(),
    };

    let executable = compile(&input);

    match executable {
        Ok(c) => write(output, c).unwrap(),
        Err(e) => println!("{e}"),
    }
}
