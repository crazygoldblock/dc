use crate::{arena::Arena, parser::Fragmenter, lexer::Tokenizer};

mod lexer;
mod parser;
mod arena;
mod ir_lowerer;
fn main() {
    println!("Hello, world!");
    
    let t = Tokenizer::new("");
    
    let mut f = Fragmenter::new(t);

    let mut arena = Arena::new();

    let f1 = f.next(&arena);

    let f2 = f1;
    
}
