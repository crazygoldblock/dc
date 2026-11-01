use crate::ir_lowerer::IRfunction;
use crate::ir_lowerer::Instruction;
use crate::ir_lowerer::IRbinaryOperator;

pub fn lower_function(function: &IRfunction) {
    let mut instructions = Vec::new();

    let mut var_count = function.paramaters.len();

    for b in function.blocks {
        for ins in b.instructions.array() {
            match ins {
                Instruction::NoOperation => panic!(),
                Instruction::Immediate { destination, value } => {
                    if *destination as usize > var_count {
                        panic!();
                    }
                    // mov rax [value]
                    instructions.extend_from_slice(&[0x48, 0xB8,  0x50]);
                    instructions.extend_from_slice(&value.to_le_bytes());
                    
                    if *destination as usize == var_count {
                        // push rax
                        instructions.push(0x50);
                        var_count += 1;
                    }
                    else {
                        // mov rsp + [8 * var_count-destination-1], rax
                        instructions.extend_from_slice(&[0x48, 0x89, 0x84, 0x24]);
                        instructions.extend_from_slice(&( 8*(var_count as u32 - (*destination as u32) - 1) ).to_le_bytes());
                    }
                },
                Instruction::ImmediateFloat { typ, destination, value } => {
                    if *destination as usize > var_count {
                        panic!();
                    }
                    if *typ == "f64" {
                        // mov rax [value]
                        instructions.extend_from_slice(&[0x48, 0xB8,  0x50]);
                        instructions.extend_from_slice(&value.to_le_bytes());
                    
                        if *destination as usize == var_count {
                            // push rax
                            instructions.push(0x50);
                            var_count += 1;
                        }
                        else {
                            // mov rsp + [8 * var_count-destination-1], rax
                            instructions.extend_from_slice(&[0x48, 0x89, 0x84, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*destination as u32) - 1) ).to_le_bytes());
                        }
                    }
                    else if *typ == "f32" {
                        // mov eax [value]
                        instructions.push(0xB8);
                        instructions.extend_from_slice(&(*value as f32).to_le_bytes());

                        if *destination as usize == var_count {
                            instructions.extend_from_slice(&[
                                // sub rsp, 8
                                0x48, 0x83, 0xEC, 0x08,
                                // mov [rsp], eax
                                0x89, 0x04, 0x24,
                            ]);
                            var_count += 1;
                        }
                        else {
                            // mov rsp + [8 * var_count-destination-1], eax
                            instructions.extend_from_slice(&[0x89, 0x84, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*destination as u32) - 1) ).to_le_bytes());
                        }
                    }
                    else { panic!(); }
                },
                Instruction::Load { size, destination, address } => todo!(),
                Instruction::Store { size, source, address } => todo!(),
                Instruction::BinaryOperation { size, operation, destination, left, right } => {
                    if operation.is_float() {
                        if *size == 32 {
                            // movss xmm0, rsp + [8 * var_count-left-1] 
                            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x84, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*left as u32) - 1) ).to_le_bytes());
                            // movss xmm1, rsp + [8 * var_count-right-1] 
                            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x8C, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*right as u32) - 1) ).to_le_bytes());
                        }
                        else if *size == 64 {
                            // movsd xmm0, rsp + [8 * var_count-left-1] 
                            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x84, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*left as u32) - 1) ).to_le_bytes());
                            // movsd xmm1, rsp + [8 * var_count-right-1] 
                            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x8C, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*right as u32) - 1) ).to_le_bytes());
                        }
                        else { panic!(); }
                    }
                    else {
                        // mov rax rsp + [8 * var_count-left-1]
                        instructions.extend_from_slice(&[0x48, 0x8B, 0x84, 0x24]);
                        instructions.extend_from_slice(&( 8*(var_count as u32 - (*left as u32) - 1) ).to_le_bytes());
                        // mov rbx rsp + [8 * var_count-right-1]
                        instructions.extend_from_slice(&[0x48, 0x8B, 0x9C, 0x24]);
                        instructions.extend_from_slice(&( 8*(var_count as u32 - (*right as u32) - 1) ).to_le_bytes());
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
                                        // mov rdx, rax
                                        0x48, 0x89, 0xC2,
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
                                        // mov dx, ax
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
                                        // mov rdx, rax
                                        0x48, 0x89, 0xC2,
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
                                        // mov dx, ax
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

                    if *destination as usize > var_count {
                        panic!();
                    }
                    if operation.is_int() || (operation.is_float() && operation.is_comparison()) {    
                        if *destination as usize == var_count {
                            // push rax
                            instructions.push(0x50);
                            var_count += 1;
                        }
                        else {
                            // mov rsp + [8 * var_count-destination-1], rax
                            instructions.extend_from_slice(&[0x48, 0x89, 0x84, 0x24]);
                            instructions.extend_from_slice(&( 8*(var_count as u32 - (*destination as u32) - 1) ).to_le_bytes());
                        }
                    }
                    else if operation.is_float() {
                        if *destination as usize == var_count {
                            // sub rsp, 8
                            instructions.extend_from_slice(&[0x48, 0x83, 0xEC, 0x08]);

                            if *size == 32 {
                                // movss [rsp], xmm0
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x11, 0x04, 0x24]);
                            }
                            else if *size == 64 {
                                // movsd [rsp], xmm0
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x11, 0x04, 0x24]);
                            }
                            var_count += 1;
                        }
                        else {
                            if *size == 32 {
                                // movss rsp + [8 * var_count-destination-1], xmm0
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x11, 0x84, 0x24]);
                                instructions.extend_from_slice(&( 8*(var_count as u32 - (*destination as u32) - 1) ).to_le_bytes());
                            }
                            else if *size == 64 {
                                // movsd rsp + [8 * var_count-destination-1], xmm0
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x11, 0x84, 0x24]);
                                instructions.extend_from_slice(&( 8*(var_count as u32 - (*destination as u32) - 1) ).to_le_bytes());
                            }
                            else { panic!(); }
                        }
                    }
                    else { panic!(); }
                },
                Instruction::Call(_) => todo!(),
                Instruction::Return { value } => todo!(),
                Instruction::Branch { condition, label } => todo!(),
                Instruction::Jump { label } => todo!(),
                Instruction::ExtendTruncate { typ, source, from_size, destination, to_size } => todo!(),
                Instruction::Convert { typ, source, from_size, destination, to_size } => todo!(),
                Instruction::Move { source, destination } => todo!(),
                Instruction::LoadGlobal {  } => todo!(),
                Instruction::StoreGlobal {  } => todo!(),
            }
        }
    }
}