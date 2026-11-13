use std::fs::read;

use crate::assembly_lowerer;
use crate::exe_builder::build;
use crate::ir_lowerer;
use crate::ir_lowerer::IRerror;
use crate::ir_lowerer::RegisterType;
use crate::{arena::Arena, lexer::Lexer, parser::parse_file};
use crate::parser::{ASTunaryOperator, ASTbinaryOperator, Fragment, Type, ParseError, ASTfunction};

pub fn compile(input_path: &str) -> Result<Vec<u8>, String> {
    let input = read(input_path).unwrap();

    let mut t = Lexer::new(&input);

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
        return Err(parse_error(fr.unwrap_err()));
    }

    let mut instructions = Vec::new();
    instructions.extend_from_slice(&START_CODE);
    instructions.extend_from_slice(&PRINT_CODE);
    instructions.extend_from_slice(&EXIT_CODE);

    let mut function_calls = Vec::new();
    
    let mut function_starts: Vec<(&[u8], usize)> = Vec::new();
    function_starts.push((b"print", 33));
    function_starts.push((b"exit", 107));

    for func in functions[2..].iter() {
        function_starts.push((func.name, instructions.len()));
        let ir = match ir_lowerer::lower_function(&arena, func, &functions) {
            Ok(ir) => ir,
            Err(e) => return Err(ir_error(e)),
        };
        assembly_lowerer::lower_function(&ir, &mut instructions, &arena, &arena, &mut function_calls);
    }

    let main = function_starts.iter().find(|x| x.0 == b"main").unwrap();
    instructions[10..14].copy_from_slice(&(main.1 as i32 - 10 - 4).to_le_bytes());

    for call in function_calls {
        let func = function_starts.iter().find(|x| x.0 == call.0).unwrap();
        instructions[call.1..call.1+4].copy_from_slice(&(func.1 as i32 - call.1 as i32 - 4).to_le_bytes());
    }

    Ok(build(&mut instructions)) 
}
fn parse_error(error: ParseError) -> String {
    match error {
        ParseError::EndOfFile => unreachable!(),
        ParseError::UnexpectedEndOfFile { line } => format!("Unexpected end of file!\nLine: {line}"),
        ParseError::ExpectedKeyword { expected, found, line } => format!("Expected keyword: {expected}, found: {found}!\nLine: {line}"),
        ParseError::ExpectedSymbol { expected, found, line } => format!("Expected symbol: \"{expected}\", found: \"{found}\"!\nLine: {line}"),
        ParseError::ExpectedIdentifier { found, line } => format!("Expected idendifier, found: {found}!\nLine:{line}"),
        ParseError::ExpectedOperator { found, line } => format!("Expected operator, found: {found}!\nLine: {line}"),
        ParseError::UnclosedBrackets { line } => format!("Unclosed brackets!\nLine: {line}"),
        ParseError::UnOpenedBrackets { line } => format!("Unopened brackets!\nLine: {line}"),
        ParseError::ExpectedConstValue { line } => format!("Expected Const value!\nLine: {line}"),
        ParseError::ExpectedNumberLiteral { line } => format!("Expected number literal!\nLine: {line}"),
    }
}
fn ir_error(error: IRerror) -> String {
    match error {
        IRerror::BinaryOperationType { operator, left, right } => {
            match operator {
                ASTbinaryOperator::Add => 
                    format!("Binary operator Addition can be used only with two integers or floats with the same sign ans size or poiner and 64bit integer!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Sub => 
                    format!("Binary operator Subtraction can be used for two types that are the same but are not pointers or bools, or pointer and i64 or u64!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Mul => 
                    format!("Binary operator Multiply can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Div => 
                    format!("Binary operator Division can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Rem => 
                    format!("Binary operator Remainder can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::And => 
                    format!("Binary operator And can be used for two types that are the same and are integers!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Xor => 
                    format!("Binary operator Xor can be used for two types that are the same and are integers!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Or => 
                    format!("Binary operator Or can be used for two types that are the same and are integers!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Shl => 
                    format!("Binary operator Shift left can be used for two types that are the same and are integers!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Shr => 
                    format!("Binary operator Shift right can be used for two types that are the same and are integers!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Less => 
                    format!("Binary operator Less can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::LessEq => 
                    format!("Binary operator Less Or Equals can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Greater => 
                    format!("Binary operator Greater can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::GreaterEq => 
                    format!("Binary operator Greater or Eguals can be used for two types that are the same and are integers or floats!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Eq => 
                    format!("Binary operator Equals can be used only for two types that are the same!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::NotEq => 
                    format!("Binary operator Not Equals can be used only for two types that are the same!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::LogicalAnd => 
                    format!("Binary operator Logical And can be used only for two bools!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::LogicalOr => 
                    format!("Binary operator Logical Or can be used only for two bools!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
                ASTbinaryOperator::Index => 
                    format!("Binary operator Index And can be used only on pointer and 64bit integer!\nLeft Type: {}, Right Type: {}", left.to_string(), right.to_string()),
            }
        },
        IRerror::UnaryOperatorType { operator, found } => {
            match operator {
                ASTunaryOperator::Sub => format!("Unary Subtraction operator can be used only with integer Types!\nFound: {}", found.to_string()),
                ASTunaryOperator::Address => panic!("no error"),
                ASTunaryOperator::Not => format!("Unary Not operator can be used only with integer Types!\nFound: {}", found.to_string()),
                ASTunaryOperator::Invert => format!("Unary Invert operator can be used only with type bool!\nFound: {}", found.to_string()),
                ASTunaryOperator::Dereference => format!("Unary Dereference operator can be used only with pointer Types!\nFound: {}", found.to_string()),
            }
        },
        IRerror::AssignmentType { expr, destination } => 
            format!("Assignment value must have the same type as the destination!\nValue Type: {}, Destination type: {}", expr.to_string(), destination.to_string()),
        IRerror::EmptyType => format!("Function call returns empty type, when value was expected"),
        IRerror::ReturnType { expected, found } => 
            format!("Return statement value has the wrong type!\nExpected: {}, Found: {}", opt_reg_str(expected), opt_reg_str(found)),
        IRerror::ConditionType { found } => 
            format!("Condition expression inside If, While, For and Do-while must return bool Type!\nfound: {}", found.to_string()),
        IRerror::ParameterType { expected, found } => 
            format!("Expression as function parameter returns wrong Type!\nExpected: {}, Found: {}", expected.to_string(), found.to_string()),
        IRerror::ArrayElementType { expected, found } => 
            format!("All elements of an array must have the same type!\nExpected: {}, Found: {}", expected.to_string(), found.to_string()),
        IRerror::CastType { exp_type, cast_type } => 
            format!("Invalid cast Type!\nFrom: {}, To: {}", exp_type.to_string(), cast_type.to_string()),
        IRerror::CallParametersCount { expected, found } => format!("Function call the wrong number of parameters!\nExpected: {expected}, Found: {found}"),
        IRerror::BreakOutsideLoop => format!("Break statement used outside of loop!"),
        IRerror::DuplicateVariableName { name } => format!("Variable with this name already exists! Name: {}", String::from_utf8_lossy(name)),
        IRerror::DeclarationDestinationType { expected, found } => 
            format!("Type of value in declaration must be the same as the variable!\nExpected: {}, Found: {}", expected.to_string(), found.to_string()),
        IRerror::LeftsideOpNotDeref => format!("The only operator allowed on the left side of assignment is Dereference!"),
        IRerror::LeftsideDerefNotVar => format!("Only variables can be dereferenced on the left side of an assignment!"),
        IRerror::LeftsideInvalidExpr => format!("Left side of assgnment statement is invalid!"),
        IRerror::VarNameNotFound { name } => format!("Use of unknown variable! Name: {}", String::from_utf8_lossy(name)),
        IRerror::FunctionNotFound { name } => format!("Use of unknown function! Name: {}", String::from_utf8_lossy(name)),
        IRerror::InvalidType { typ } => format!("Found invalid Type! Type: {}", String::from_utf8_lossy(typ)),
        IRerror::MissignReturn { function_name } => format!("Missing return statement inside function! Function name: {}", String::from_utf8_lossy(function_name)),
        IRerror::ContinueOutsideLoop => format!("Continue statement used outside loop!"),
    }
}
fn opt_reg_str(reg: Option<RegisterType>) -> String {
    match reg {
        Some(r) => r.to_string(),
        None => "None".to_string(),
    }
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