use std::{collections::HashMap, fs::{read_to_string, write}};

use crate::{arena::Arena, ir_lowerer::lower_function, lexer::Tokenizer, parser::{Fragment, Fragmenter}};

#[test] 
fn test1() {
    let file = read_to_string("./inputs/first.dl").unwrap();
    let t = Tokenizer::new(file.as_bytes());
    
    println!("{}", &file);

    let mut f = Fragmenter::new(t);

    let mut arena = Arena::new();

    let f1 = f.next(&arena).unwrap();

    let functions = HashMap::new();

    if let Fragment::Function(f) = f1 {
        println!("AST FUNCTION: {:?}", f);
        let cf = lower_function(&arena, &f, &functions).unwrap();

        let res = format!("{:?}", cf);

        write("./inputs/output.txt", res).unwrap();
    }

    

    
}