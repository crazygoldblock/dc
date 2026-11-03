#[test] 
fn check_ast_and_ir() {
    use std::fs::{read_dir, read_to_string, write, exists, read};

    use crate::arena::Arena;
    use crate::ir_lowerer::lower_function;
    use crate::lexer::Tokenizer;
    use crate::parser::{Fragment, parse_file, ParseError};
    use crate::assembly_lowerer;

    for entry in read_dir("./inputs").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "dl" {
                    let file = read_to_string(&path).unwrap();
                    let mut tokenizer = Tokenizer::new(file.as_bytes());
                    let arena = Arena::new();

                    let mut ast = String::new();
                    let mut ir = String::new();
                    let mut instructions = Vec::new();

                    let mut frag = parse_file(&mut tokenizer, &arena);
                    while let Ok(fr) = frag {
                        ast += &format!("{:?}", fr);

                        if let Fragment::ASTfunction(func) = fr {
                            let cf = lower_function(&arena, &func).unwrap();
                            ir += &format!("{:?}", cf);
                            assembly_lowerer::lower_function(&cf, &mut instructions, &arena).unwrap();
                        }
                        frag = parse_file(&mut tokenizer, &arena);
                    }
                    if !matches!(frag, Err(ParseError::EndOfFile)) {
                        frag.unwrap();
                    }
                    let output_path = format!("./inputs/{}", path.file_stem().unwrap().to_str().unwrap());

                    if exists(format!("{}.ast", output_path)).unwrap() {
                        let ast_file = read_to_string(format!("{}.ast", output_path)).unwrap();
                        if ast_file != format!("{}", ast) {
                            panic!("AST comparison failed! FUNCTION: \n{}\nPATH: {}, \n\nOLD: \n{}\n\nNEW: \n{}\n", file, path.to_str().unwrap(), ast_file, ast);
                        }
                    }
                    if exists(format!("{}.ir", output_path)).unwrap() {
                        let ir_file = read_to_string(format!("{}.ir", output_path)).unwrap();
                        if ir_file != format!("{}", ir) {
                            panic!("IR comparison failed! FUNCTION: \n{}\nPATH: {}, \n\nOLD: \n{}\n\nNEW: \n{}\n", file, path.to_str().unwrap(), ir_file, ir);
                        }
                    }
                    if exists(format!("{}.asm", output_path)).unwrap() {
                        let asm_file = read(format!("{}.asm", output_path)).unwrap();
                        if asm_file != instructions {
                            panic!("ASM comparison failed! FUNCTION: \n{}\nPATH: {}, \n\nOLD: \n{:?}\n\nNEW: \n{:?}\n", file, path.to_str().unwrap(), asm_file, instructions);
                        }
                    }
                    
                    write(format!("{}.ast", output_path), ast).unwrap();
                    write(format!("{}.ir", output_path), ir).unwrap();
                    write(format!("{}.asm", output_path), instructions).unwrap();
                }     
            }
        }
    }
}
