#[test] 
fn check_ast_and_ir() {
    use std::collections::HashMap;
    use std::fs::{read_dir, read_to_string, write, exists};

    use crate::{arena::Arena, ir_lowerer::lower_function, lexer::Tokenizer, parser::{Fragment, Fragmenter}};

    for entry in read_dir("./inputs").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "dl" {
                    let file = read_to_string(&path).unwrap();
                    let t = Tokenizer::new(file.as_bytes());
                    let mut f = Fragmenter::new(t);
                    let arena = Arena::new();

                    let mut ast = String::new();
                    let mut ir = String::new();

                    let mut frag = f.next(&arena);
                    while let Some(fr) = frag {
                        ast += &format!("{:?}", fr);

                        if let Fragment::ASTfunction(func) = fr {
                            let functions = HashMap::new();
                            let cf = lower_function(&arena, &func, &functions).unwrap();
                            ir += &format!("{:?}", cf);
                        }
                        frag = f.next(&arena);
                    }
                    let output_path = format!("./inputs/{}", path.file_stem().unwrap().to_str().unwrap());

                    if exists(format!("{}.ast", output_path)).unwrap() {
                        let ast_file = read_to_string(format!("{}.ast", output_path)).unwrap();
                        if ast_file != format!("{}", ast) {
                            panic!("AST comparison failed! Path: {}, Old: \n{}\n New: \n{}\n", path.to_str().unwrap(), ast_file, ast);
                        }
                    }
                    if exists(format!("{}.ir", output_path)).unwrap() {
                        let ir_file = read_to_string(format!("{}.ir", output_path)).unwrap();
                        if ir_file != format!("{}", ir) {
                            panic!("IR comparison failed! Path: {}, Old: \n{}\n New: \n{}\n", path.to_str().unwrap(), ir_file, ir);
                        }
                    }
                    
                    write(format!("{}.ast", output_path), ast).unwrap();
                    write(format!("{}.ir", output_path), ir).unwrap();
                }     
            }
        }
    }
}
