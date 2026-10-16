use crate::{arena::Arena, parser::Fragmenter, tokenizer::Tokenizer};

mod tokenizer;
mod parser;
mod arena;
mod low_ir;
fn main() {
    println!("Hello, world!");
    
    let t = Tokenizer::new("");
    
    let mut f = Fragmenter::new(t);

    let mut arena = Arena::new();

    let f1 = f.next(&arena);

    let f2 = f1;
    
}
