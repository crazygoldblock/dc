#![allow(unused)]

use std::fs::read;

use crate::assembly_lowerer;
use crate::ir_lowerer;
use crate::{arena::Arena, lexer::Tokenizer, parser::parse_file};
use crate::parser::Fragment;

pub fn compile(input_path: &str, output_path: &str) {
    let input = read(input_path).unwrap();

    let mut t = Tokenizer::new(&input);

    let mut arena = Arena::new();

    let mut instructions = Vec::new();
    let mut function_calls = Vec::new();

    let mut fr = parse_file(&mut t, &arena);
    while let Ok(f) = fr {
        match f {
            Fragment::ASTfunction(astfunction) => {
                let f2 = ir_lowerer::lower_function(&arena, &astfunction).unwrap();
                let f3 = assembly_lowerer::lower_function(&f2, &mut instructions, &arena).unwrap();
                function_calls.push(f3);
            },
            Fragment::Include(include) => todo!(),
            Fragment::Constant(constant) => todo!(),
            Fragment::Struct(_) => todo!(),
            Fragment::Union(union) => todo!(),
            Fragment::GlobalVariable(global_variable) => todo!(),
        }
        fr = parse_file(&mut t, &arena);
    }
    
}