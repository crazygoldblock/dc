use crate::tokenizer::Tokenizer;


mod tokenizer;
mod parser;
mod arena;
fn main() {
    println!("Hello, world!");
    let mut t = Tokenizer::new("");
    t.next();

    let mut vec = Vec::new();
    for i in 0..1000 {
        vec.push(t.next().unwrap());
    }


    print!("{}", vec[0]);
}
