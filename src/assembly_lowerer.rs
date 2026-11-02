#![allow(unused)]

use std::collections::HashMap;

use crate::ir_lowerer::IRfunction;
use crate::ir_lowerer::Instruction;
use crate::ir_lowerer::IRbinaryOperator;
use crate::ir_lowerer::ExtTruncType;
use crate::ir_lowerer::ConvertType;

// rbp = start rsp

// TODO: call offset patching

pub fn lower_function<'a>(
    function: &IRfunction,
    functions: &HashMap<&'a [u8], u32>,
    instructions: &mut Vec<u8>,
) {
    let mut var_count = function.paramaters.len();

    let mut jumps = Vec::new();
    let mut labels = Vec::new();

    instructions.extend_from_slice(&[
        // mov rbp, rsp
        0x48, 0x89, 0xE5,
        // sub rbp, [8*parameter_count]
        0x48, 0x81, 0xED
    ]);
    instructions.extend_from_slice(&(function.paramaters.len() as u32 * 8).to_le_bytes());

    for b in function.blocks {
        labels.push(instructions.len() as u32);
        for ins in b.instructions.array() {
            match ins {
                Instruction::NoOperation => panic!(),
                Instruction::Immediate { destination, value } => {
                    // mov rax [value]
                    instructions.extend_from_slice(&[0x48, 0xB8,  0x50]);
                    instructions.extend_from_slice(&value.to_le_bytes());
                    
                    store(instructions, *destination, Register::Rax, &mut var_count);
                },
                Instruction::ImmediateFloat { typ, destination, value } => {
                    if *destination as usize > var_count {
                        panic!();
                    }
                    if *typ == "f64" {
                        // mov rax [value]
                        instructions.extend_from_slice(&[0x48, 0xB8,  0x50]);
                        instructions.extend_from_slice(&value.to_le_bytes());
                    
                        store(instructions, *destination, Register::Rax, &mut var_count);
                    }
                    else if *typ == "f32" {
                        // mov eax [value]
                        instructions.push(0xB8);
                        instructions.extend_from_slice(&(*value as f32).to_le_bytes());

                        store(instructions, *destination, Register::Rax, &mut var_count);
                    }
                    else { panic!(); }
                },
                Instruction::Load { size, destination, address } => todo!(),
                Instruction::Store { size, source, address } => todo!(),
                Instruction::BinaryOperation { size, operation, destination, left, right } => {
                    if operation.is_float() {
                        if *size == 32 {
                            load(instructions, *left, Register::Xmm0, var_count);
                            load(instructions, *right, Register::Xmm1, var_count);
                        }
                        else if *size == 64 {
                            load(instructions, *left, Register::Xmm0d, var_count);
                            load(instructions, *right, Register::Xmm1d, var_count);
                        }
                        else { panic!(); }
                    }
                    else {
                        load(instructions, *left, Register::Rax, var_count);
                        load(instructions, *right, Register::Rbx, var_count);
                    }

                    match operation {
                        IRbinaryOperator::Add => {
                            // add rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x01, 0xD8]),
                                32 => instructions.extend_from_slice(&[0x01, 0xD8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x01, 0xD8]),
                                8 => instructions.extend_from_slice(&[0x00, 0xD8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Sub => {
                            // sub rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x29, 0xD8]),
                                32 => instructions.extend_from_slice(&[0x29, 0xD8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x29, 0xD8]),
                                8 => instructions.extend_from_slice(&[0x28, 0xD8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Mul => {
                            // imul rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x0F, 0xAF, 0xC3]),
                                32 => instructions.extend_from_slice(&[0x0F, 0xAF, 0xC3]),
                                16 => instructions.extend_from_slice(&[0x66, 0x0F, 0xAF, 0xC3]),
                                8 => instructions.extend_from_slice(&[0xF6, 0xEB]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Udiv => {
                            match size {
                                64 => { 
                                    instructions.extend_from_slice(&[
                                        // xor rdx, rdx
                                        0x48, 0x31, 0xD2,
                                        // div rbx
                                        0x48, 0xF7, 0xF3,
                                    ]);
                                },
                                32 => {
                                    instructions.extend_from_slice(&[
                                        // xor edx, edx
                                        0x31, 0xD2,
                                        // div ebx
                                        0xF7, 0xF3,
                                    ]);
                                },
                                16 => {
                                    instructions.extend_from_slice(&[
                                        // xor dx, dx
                                        0x66, 0x31, 0xD2,
                                        // div bx
                                        0x66, 0xF7, 0xF3,
                                    ]);
                                },
                                8 => {
                                    instructions.extend_from_slice(&[
                                        // xor ah, ah
                                        0x30, 0xE4,
                                        // div bl
                                        0xF6, 0xF3,
                                    ]);
                                },
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Sdiv => {
                            match size {
                                64 => { 
                                    instructions.extend_from_slice(&[
                                        // cqo
                                        0x48, 0x99,
                                        // idiv rbx
                                        0x48, 0xF7, 0xFB,
                                    ]);
                                },
                                32 => {
                                    instructions.extend_from_slice(&[
                                        // cdq
                                        0x99,
                                        // idiv ebx
                                        0xF7, 0xFB,
                                    ]);
                                },
                                16 => {
                                    instructions.extend_from_slice(&[
                                        // cwd
                                        0x66, 0x99,
                                        // idiv bx
                                        0x66, 0xF7, 0xFB,
                                    ]);
                                },
                                8 => {
                                    instructions.extend_from_slice(&[
                                        // cbw
                                        0x66, 0x98,
                                        // idiv bl
                                        0xF6, 0xFB,
                                    ]);
                                },
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Urem => {
                            match size {
                                64 => { 
                                    instructions.extend_from_slice(&[
                                        // xor rdx, rdx
                                        0x48, 0x31, 0xD2,
                                        // div rbx
                                        0x48, 0xF7, 0xF3,
                                        // mov rax, rdx
                                        0x48, 0x89, 0xD0,
                                    ]);
                                },
                                32 => {
                                    instructions.extend_from_slice(&[
                                        // xor edx, edx
                                        0x31, 0xD2,
                                        // div ebx
                                        0xF7, 0xF3,
                                        // mov eax, edx
                                        0x89, 0xD0,
                                    ]);
                                },
                                16 => {
                                    instructions.extend_from_slice(&[
                                        // xor dx, dx
                                        0x66, 0x31, 0xD2,
                                        // div bx
                                        0x66, 0xF7, 0xF3,
                                        // mov ax, dx
                                        0x66, 0x89, 0xD0,
                                    ]);
                                },
                                8 => {
                                    instructions.extend_from_slice(&[
                                        // xor ah, ah
                                        0x30, 0xE4,
                                        // div bl
                                        0xF6, 0xF3,
                                        // mov al, dl
                                        0x88, 0xD0,
                                    ]);
                                },
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Srem => {
                            match size {
                                64 => { 
                                    instructions.extend_from_slice(&[
                                        // cqo
                                        0x48, 0x99,
                                        // idiv rbx
                                        0x48, 0xF7, 0xFB,
                                        // mov rax, rdx
                                        0x48, 0x89, 0xD0,
                                    ]);
                                },
                                32 => {
                                    instructions.extend_from_slice(&[
                                        // cdq
                                        0x99,
                                        // idiv ebx
                                        0xF7, 0xFB,
                                        // mov eax, edx
                                        0x89, 0xD0,
                                    ]);
                                },
                                16 => {
                                    instructions.extend_from_slice(&[
                                        // cwd
                                        0x66, 0x99,
                                        // idiv bx
                                        0x66, 0xF7, 0xFB,
                                        // mov ax, dx
                                        0x66, 0x89, 0xD0,
                                    ]);
                                },
                                8 => {
                                    instructions.extend_from_slice(&[
                                        // cbw
                                        0x66, 0x98,
                                        // idiv bl
                                        0xF6, 0xFB,
                                        // mov al. dl
                                        0x88, 0xD0,
                                    ]);
                                },
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::And => {
                            // and rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x21, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x21, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x21, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x20, 0xd8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Or => {
                            // or rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x09, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x09, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x09, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x08, 0xd8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Xor => {
                            // xor rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x31, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x31, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x31, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x30, 0xd8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Shl => {
                            // mov rcx, rbx
                            instructions.extend_from_slice(&[0x48, 0x89, 0xd9]);
                            // shl rax cl
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0xD3, 0xE0]),
                                32 => instructions.extend_from_slice(&[0xD3, 0xE0]),
                                16 => instructions.extend_from_slice(&[0x66, 0xD3, 0xE0]),
                                8 => instructions.extend_from_slice(&[0xD2, 0xE0]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::ShrZeroFill => {
                            // mov rcx, rbx
                            instructions.extend_from_slice(&[0x48, 0x89, 0xd9]);
                            // shr rax cl
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0xD3, 0xE8]),
                                32 => instructions.extend_from_slice(&[0xD3, 0xE8]),
                                16 => instructions.extend_from_slice(&[0x66, 0xD3, 0xE8]),
                                8 => instructions.extend_from_slice(&[0xD2, 0xE8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::ShrSignFill => {
                            // mov rcx, rbx
                            instructions.extend_from_slice(&[0x48, 0x89, 0xd9]);
                            // sar rax cl
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0xD3, 0xF8]),
                                32 => instructions.extend_from_slice(&[0xD3, 0xF8]),
                                16 => instructions.extend_from_slice(&[0x66, 0xD3, 0xF8]),
                                8 => instructions.extend_from_slice(&[0xD2, 0xF8]),
                                _ => panic!(),
                            }
                        },
                        IRbinaryOperator::Eq => {
                            instructions.extend_from_slice(&[
                                // cmp rax, rbx    
                                0x48, 0x39, 0xD8,
                                // sete al        
                                0x0F, 0x94, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::NotEq => {
                            instructions.extend_from_slice(&[
                                // cmp rax, rbx    
                                0x48, 0x39, 0xD8,
                                // setne al        
                                0x0F, 0x95, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Ugreater => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // seta al        
                                0x0F, 0x97, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::UgreaterEq => {
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setae al        
                                0x0F, 0x93, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Uless => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setb al        
                                0x0F, 0x92, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::UlessEq => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setbe al        
                                0x0F, 0x96, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Sgreater => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setg al        
                                0x0F, 0x9F, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::SgreaterEq => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setge al        
                                0x0F, 0x9D, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Sless => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setl al        
                                0x0F, 0x9C, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::SlessEq => {
                            // cmp rax, rbx
                            match size {
                                64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                                _ => panic!(),
                            }
                            instructions.extend_from_slice(&[
                                // setle al        
                                0x0F, 0x9E, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Feq => {
                            if *size == 32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == 64 {
                                // ucomisd xmm0, xmm1
                                instructions.extend_from_slice(&[0x66, 0x0F, 0x2E, 0xC1]);
                            }
                            else { panic!(); }
                            instructions.extend_from_slice(&[
                                // sete al        
                                0x0F, 0x94, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::FnotEq => {
                            if *size == 32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == 64 {
                                // ucomisd xmm0, xmm1
                                instructions.extend_from_slice(&[0x66, 0x0F, 0x2E, 0xC1]);
                            }
                            else { panic!(); }
                            instructions.extend_from_slice(&[
                                // setne al        
                                0x0F, 0x95, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Fgreater => {
                            if *size == 32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == 64 {
                                // ucomisd xmm0, xmm1
                                instructions.extend_from_slice(&[0x66, 0x0F, 0x2E, 0xC1]);
                            }
                            else { panic!(); }
                            instructions.extend_from_slice(&[
                                // seta al        
                                0x0F, 0x97, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::FgreaterEq => {
                            if *size == 32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == 64 {
                                // ucomisd xmm0, xmm1
                                instructions.extend_from_slice(&[0x66, 0x0F, 0x2E, 0xC1]);
                            }
                            else { panic!(); }
                            instructions.extend_from_slice(&[
                                // setge al        
                                0x0F, 0x9D, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Fless => {
                            if *size == 32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == 64 {
                                // ucomisd xmm0, xmm1
                                instructions.extend_from_slice(&[0x66, 0x0F, 0x2E, 0xC1]);
                            }
                            else { panic!(); }
                            instructions.extend_from_slice(&[
                                // setb al        
                                0x0F, 0x92, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::FlessEq => {
                            if *size == 32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == 64 {
                                // ucomisd xmm0, xmm1
                                instructions.extend_from_slice(&[0x66, 0x0F, 0x2E, 0xC1]);
                            }
                            else { panic!(); }
                            instructions.extend_from_slice(&[
                                // setbe al        
                                0x0F, 0x96, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Fadd => {
                            if *size == 32 {
                                // addss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x58, 0xC1]);
                            }
                            else if *size == 64 {
                                // addsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x58, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Fsub => {
                            if *size == 32 {
                                // subss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x5C, 0xC1]);
                            }
                            else if *size == 64 {
                                // subsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x5C, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Fmul => {
                            if *size == 32 {
                                // mulss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x59, 0xC1]);
                            }
                            else if *size == 64 {
                                // mulsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x59, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Fdiv => {
                            if *size == 32 {
                                // divss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x5E, 0xC1]);
                            }
                            else if *size == 64 {
                                // divsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x5E, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Frem => todo!(),
                    }
                    if operation.is_int() || (operation.is_float() && operation.is_comparison()) {    
                        store(instructions, *destination, Register::Rax, &mut var_count);
                    }
                    else if operation.is_float() {
                        if *size == 32 {
                            store(instructions, *destination, Register::Xmm0, &mut var_count);
                        }
                        else if *size == 64 {
                            store(instructions, *destination, Register::Xmm0d, &mut var_count);
                        }
                        else { panic!(); }
                    }
                    else { panic!(); }
                },
                Instruction::Call((name, parameters, ret)) => {
                    for par in *parameters {
                        load(instructions, *par, Register::Rax, var_count);
                        // push rax
                        instructions.push(0x50);
                    }
                    // call [rip + offset]
                    instructions.extend_from_slice(&[0xFF, 0x15]);
                    let offset = instructions.len() as u32 - functions.get(name).unwrap() - 6;
                    instructions.extend_from_slice(&offset.to_le_bytes());

                    if let Some(ret) = ret {
                        // pop rax
                        instructions.push(0x58);
                        store(instructions, *ret, Register::Rax, &mut var_count);
                    }
                },
                Instruction::Return { value } => {
                    match value {
                        Some(value) => {
                            load(instructions, *value, Register::Rax, var_count);
                            instructions.extend_from_slice(&[
                                // mov rsp, rbp
                                0x48, 0x89, 0xEC,
                                // push rax
                                0x50,
                                // ret
                                0xC3,
                            ]);
                            
                        },
                        None => {
                            instructions.extend_from_slice(&[
                                // mov rsp, rbp
                                0x48, 0x89, 0xEC,
                                // ret
                                0xC3,
                            ]);
                        },
                    }
                },
                Instruction::Branch { condition, label } => {
                    load(instructions, *condition, Register::Rax, var_count);
                    instructions.extend_from_slice(&[
                        // cmp rax, 1
                        0x48, 0x83, 0xF8, 0x01,
                        // jne 5
                        0x75, 0x05,
                        // jmp [offset]
                        0xE9,
                    ]);
                    jumps.push((instructions.len(), *label));
                    instructions.extend_from_slice(&0u32.to_le_bytes());
                },
                Instruction::Jump { label } => {
                    // jmp [offset] 
                    instructions.extend_from_slice(&[0xE9]);

                    jumps.push((instructions.len(), *label));
                    instructions.extend_from_slice(&0u32.to_le_bytes());
                },
                Instruction::ExtendTruncate { typ, source, from_size, destination, to_size } => {
                    match typ {
                        ExtTruncType::ZeroExtend => {
                            load(instructions, *source, Register::Rax, var_count);
                            match (from_size, to_size) {
                                // movsx ax, al
                                (8, 16) => instructions.extend_from_slice(&[0x66, 0x0F, 0xB6, 0xC0]),
                                (8, 32) => instructions.extend_from_slice(&[0x0F, 0xB6, 0xC0]),
                                (8, 64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]),
                                (16, 32) => instructions.extend_from_slice(&[0x0F, 0xB7, 0xC0]),
                                (16, 64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0xC0]),
                                (32, 64) => instructions.extend_from_slice(&[0x89, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Rax, &mut var_count);
                        },
                        ExtTruncType::SignExtend => {
                            load(instructions, *source, Register::Rax, var_count);
                            match (from_size, to_size) {
                                // movsx ax, al
                                (8, 16) => instructions.extend_from_slice(&[0x66, 0x0F, 0xBE, 0xC0]),
                                (8, 32) => instructions.extend_from_slice(&[0x0F, 0xBE, 0xC0]),
                                (8, 64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xBE, 0xC0]),
                                (16, 32) => instructions.extend_from_slice(&[0x0F, 0xBF, 0xC0]),
                                (16, 64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xBF, 0xC0]),
                                (32, 64) => instructions.extend_from_slice(&[0x48, 0x63, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Rax, &mut var_count);
                        },
                        ExtTruncType::Fextend => {
                            load(instructions, *source, Register::Xmm0, var_count);
                            match (from_size, to_size) {
                                // cvtss2sd xmm0, xmm0
                                (32, 64) => instructions.extend_from_slice(&[0xF3, 0x0F, 0x5A, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Xmm0d, &mut var_count);
                        },
                        ExtTruncType::Truncate => {
                            load(instructions, *source, Register::Rax, var_count);
                            store(instructions, *destination, Register::Rax, &mut var_count);
                        },
                        ExtTruncType::Ftruncate => {
                            load(instructions, *source, Register::Xmm0d, var_count);
                            match (from_size, to_size) {
                                // cvtsd2ss xmm0, xmm0
                                (64, 32) => instructions.extend_from_slice(&[0xF2, 0x0F, 0x5A, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Xmm0, &mut var_count);
                        },
                    }
                },
                Instruction::Convert { typ, source, from_size, destination, to_size } => {
                    match typ {
                        ConvertType::FloatToUnsigned | ConvertType::FloatToSigned => {
                            if  *from_size == 32 {
                                load(instructions, *source, Register::Xmm0, var_count);
                                // cvttss2si rax, xmm0
                                instructions.extend_from_slice(&[0xF3, 0x48, 0x0F, 0x2C, 0xC0]);
                            }
                            else if *from_size == 64 {
                                load(instructions, *source, Register::Xmm0d, var_count);
                                // cvttsd2si rax, xmm0
                                instructions.extend_from_slice(&[0xF2, 0x48, 0x0F, 0x2C, 0xC0]);
                            }
                            else { panic!(); }

                            store(instructions, *destination, Register::Rax, &mut var_count);
                        },
                        ConvertType::SignedToFloat | ConvertType::UnsignedToFloat => {
                            load(instructions, *source, Register::Rax, var_count);
                            if  *to_size == 32 {
                                // cvtsi2ss xmm0, rax
                                instructions.extend_from_slice(&[0xF3, 0x48, 0x0F, 0x2A, 0xC0]);
                                store(instructions, *destination, Register::Xmm0, &mut var_count);
                            }
                            else if *from_size == 64 {
                                // cvtsi2sd xmm0, rax
                                instructions.extend_from_slice(&[0xF2, 0x48, 0x0F, 0x2A, 0xC0]);
                                store(instructions, *destination, Register::Xmm0d, &mut var_count);
                            }
                            else { panic!(); }
                        },
                    }
                },
                Instruction::Move { source, destination } => {
                    load(instructions, *source, Register::Rax, var_count);
                    store(instructions, *destination, Register::Rax, &mut var_count);
                },
                Instruction::LoadGlobal {  } => todo!(),
                Instruction::StoreGlobal {  } => todo!(),
            }
        }
    }

    if function.return_type.is_some()  {
        if !matches!(function.blocks[0].instructions.array()[0], Instruction::Return{value: _}) {
            panic!();
        }
    }
    else {
        instructions.extend_from_slice(&[
            // mov rsp, rbp
            0x48, 0x89, 0xEC,
            // ret
            0xC3,
        ]);
    }

    for (jmp, label) in jumps {
        let offset = labels[label as usize].to_le_bytes();
        instructions[jmp..].copy_from_slice(&offset);
    }

}
fn load(instructions: &mut Vec<u8>, source: u16, register: Register, var_count: usize) {
    if source as usize >= var_count {
        panic!();
    }
    match register {
        Register::Rax => {
            // mov rax, rbp + [-8 * index+1]
            instructions.extend_from_slice(&[0x48, 0x8B, 0x85]);
        },
        Register::Rbx => {
            // mov rbx, rbp + [-8 * index+1]
            instructions.extend_from_slice(&[0x48, 0x8B, 0x9D]);
        },
        Register::Xmm0 => {
            // movss xmm0, rbp + [-8 * left+1] 
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x85]);               
        },
        Register::Xmm1 => {
            // movss xmm1, rbp + [-8 * right+1] 
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x8D]);
        },
        Register::Xmm0d => {
            // movsd xmm0, rbp + [-8 * left+1] 
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x85]);             
        },
        Register::Xmm1d => {
            // movsd xmm1, rbp + [-8 * right+1] 
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x8D]);
        },
    }
    instructions.extend_from_slice(&( -8*(source as i32 + 1) ).to_le_bytes());
}
fn store(instructions: &mut Vec<u8>, destination: u16, register: Register, var_count: &mut usize) {
    if destination as usize > *var_count {
        panic!();
    }
    if destination as usize == *var_count {
        // sub rsp, 8
        instructions.extend_from_slice(&[0x48, 0x83, 0xEC, 0x08]);
        *var_count += 1;
    }
    match register {
        Register::Rax => {
            // mov rsp + [-8 * destination+1], rax
            instructions.extend_from_slice(&[0x48, 0x89, 0x85]);
        },
        Register::Xmm0 => {
            // movss rbp + [-8 * destination+1], xmm0
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x11, 0x85]);
        },
        Register::Xmm0d => {
            // movsd rbp + [-8 * destination+1], xmm0
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x11, 0x85]);
        },
        _ => panic!(),
    }
    instructions.extend_from_slice(&( -8*(destination as i32 + 1) ).to_le_bytes());
}
enum Register {
    Rax,
    Rbx,
    Xmm0,
    Xmm1,
    Xmm0d,
    Xmm1d,
}