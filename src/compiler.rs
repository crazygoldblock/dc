use std::fs::read;

use crate::assembly_lowerer;
use crate::exe_builder::build;
use crate::ir_lowerer;
use crate::parser::ASTfunction;
use crate::parser::ParseError;
use crate::parser::Type;
use crate::{arena::Arena, lexer::Tokenizer, parser::parse_file};
use crate::parser::Fragment;

pub fn compile(input_path: &str, output_path: &str) {
    let input = read(input_path).unwrap();

    let mut t = Tokenizer::new(&input);

    let arena = Arena::new();

    let mut functions: Vec<ASTfunction<'_, '_>> = Vec::new();

    let pars: &'static [(Type<'static, 'static>, &'static [u8])] = &[ (Type::Pointer(&Type::Name(b"u8")), b""), (Type::Name(b"u64"), b"") ];
    functions.push(ASTfunction { public: true, return_type: None, name: b"print", parameters: pars, body: &[] });

    let pars: &'static [(Type<'static, 'static>, &'static [u8])] = &[ (Type::Name(b"i32"), b"") ];
    functions.push(ASTfunction { public: true, return_type: None, name: b"exit", parameters: pars, body: &[] });

    let mut fr = parse_file(&mut t, &arena);
    while let Ok(f) = fr {
        match f {
            Fragment::ASTfunction(astfunction) => {
                if functions.iter().find(|x| x.name == astfunction.name).is_some() {
                    panic!();
                }
                if astfunction.name == b"main" {
                    if astfunction.return_type != Some(Type::Name(b"i32")) {
                        panic!();
                    }
                    if astfunction.parameters.len() != 1 {
                        panic!();
                    }
                    if astfunction.parameters[0].0 != Type::Pointer(&Type::Name(b"u8")) {
                        panic!();
                    }
                }
                functions.push(astfunction);
            },
            Fragment::Include(_) => todo!(),
            Fragment::Constant(_) => todo!(),
            Fragment::Struct(_) => todo!(),
            Fragment::Union(_) => todo!(),
            Fragment::GlobalVariable(_) => todo!(),
        }
        fr = parse_file(&mut t, &arena);
    }
    if !matches!(fr, Err(ParseError::EndOfFile)) {
        fr.unwrap();
    }

    let mut instructions = Vec::new();
    instructions.extend_from_slice(&START_CODE);
    instructions.extend_from_slice(&PRINT_CODE);
    instructions.extend_from_slice(&EXIT_CODE);

    // user code calling system func

    let mut function_calls = Vec::new();
    
    let mut function_starts: Vec<(&[u8], usize)> = Vec::new();
    function_starts.push((b"print", 33));
    function_starts.push((b"exit", 107));

    for func in functions[2..].iter() {
        function_starts.push((func.name, instructions.len()));
        let ir = ir_lowerer::lower_function(&arena, func, &functions).unwrap();
        assembly_lowerer::lower_function(&ir, &mut instructions, &arena, &mut function_calls);
    }

    let main = function_starts.iter().find(|x| x.0 == b"main").unwrap();
    instructions[10..14].copy_from_slice(&(main.1 as i32 - 10 - 4).to_le_bytes());

    for call in function_calls {
        let func = function_starts.iter().find(|x| x.0 == call.0).unwrap();
        instructions[call.1..call.1+4].copy_from_slice(&(func.1 as i32 - call.1 as i32 - 4).to_le_bytes());
    }

    build(&mut instructions, output_path);
    
}

const START_CODE: [u8; 33] = [
    // call [rip - 0x1000 - 6 + 104] command line FFFF F062
    0xFF, 0x15, 0x62, 0xF0, 0xFF, 0xFF,
    // push rax
    0x50,
    // lea rax, [rip + offset]
    0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
    // call rax
    0xFF, 0xD0,
    // mov rcx, rax
    0x48, 0x89, 0xC1,
    // and rsp, -0x10
    0x48, 0x83, 0xE4, 0xF0,
    // sub rsp, 0x20
    0x48, 0x83, 0xEC, 0x20,
    // call [rip - 0x1000 - 33 + 96] exit FFFF F03F
    0xFF, 0x15, 0x3F, 0xF0, 0xFF, 0xFF
];
const PRINT_CODE: [u8; 74] = [
    // mov rcx, rsp
    0x48, 0x89, 0xE1,
    // mov rax, [rsp+0x8]
    0x48, 0x8B, 0x44, 0x24, 0x08,
    // mov rdx, [rsp+0x10]
    0x48, 0x8B, 0x54, 0x24, 0x10,
    // and rsp, -0x10
    0x48, 0x83, 0xE4, 0xF0,
    // push rcx
    0x51,
    // push rax
    0x50,
    // push rdx
    0x52,
    // sub rsp, 0x38
    0x48, 0x83, 0xEC, 0x38,

    // mov ecx, 0xfffffff5
	0xB9, 0xF5, 0xFF, 0xFF, 0xFF,	
    // call *[rip - 4096 - 35 - 33 + 80] getStdHandle FFFF F00C
    0xff, 0x15, 0x0C, 0xF0, 0xFF, 0xFF,

    // mov rcx, rax
	0x48, 0x89, 0xc1, 
    // mov rdx, [rsp+0x38]
	0x48, 0x8b, 0x54, 0x24, 0x38,
    // mov r8, [rsp+0x40]
	0x4c, 0x8b, 0x44, 0x24, 0x40,
    // lea r9, [rsp+0x28]
	0x4c, 0x8d, 0x4c, 0x24, 0x28,	
    // mov qword [rsp+0x20], 0x0
	0x48, 0xc7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00,
    // call *[rip - 4096 - 68 - 33 + 88] writeFile FFFF EFF3
	0xff, 0x15, 0xF3, 0xEF, 0xFF, 0xFF,
    // add rsp, 0x48
	0x48, 0x83, 0xc4, 0x48,
    // pop rsp
    0x5C,
    // ret	
	0xc3,
];
const EXIT_CODE: [u8; 19] = [
    // mov rcx, [rsp + 0x8]
    0x48, 0x8B, 0x4C, 0x24, 0x08,
    // and rsp, -0x10
    0x48, 0x83, 0xE4, 0xF0,
    // sub rsp, 0x20
    0x48, 0x83, 0xEC, 0x20,
    // call *[rip - 4096 - 33 - 74 - 19 + 96] exit FFFF EFE2
    0xff, 0x15, 0xE2, 0xEF, 0xFF, 0xFF,
];
//const READ_CODE: &[u8] = &[
    
//];
//80 gethandle
//88 write
//96 exit
//104 command line