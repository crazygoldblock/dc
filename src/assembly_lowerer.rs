use crate::arena::Arena;
use crate::ir_lowerer::IRfunction;
use crate::ir_lowerer::Instruction;
use crate::ir_lowerer::IRbinaryOperator;
use crate::ir_lowerer::ExtTruncType;
use crate::ir_lowerer::ConvertType;

pub fn lower_function<'b>(
    function: &IRfunction,
    instructions: &mut Vec<u8>,
    arena: &'b Arena,
    temp: &Arena,
    function_calls: &mut Vec<(&'b [u8], usize)>,
) {
    let stack_size = function.paramaters_count as u32 * 8 + 8;

    let mut jumps = temp.vec_capacity(16);
    let mut labels = temp.vec_capacity(16);

    instructions.extend_from_slice(&[
        // mov rbp, rsp
        0x48, 0x89, 0xE5,
        // add rbp, [stack_size]
        0x48, 0x81, 0xC5
    ]);
    instructions.extend_from_slice(&stack_size.to_le_bytes());

    // sub rsp, [stack_size]
    instructions.extend_from_slice(&[0x48, 0x81, 0xEC]);
    instructions.extend_from_slice(&(function.stack_size - stack_size).to_le_bytes());

    for b in function.blocks {
        labels.push(instructions.len() as u32);
        for ins in b.instructions.array() {
            match ins {
                Instruction::Immediate { destination, value } => {
                    // mov rax [value]
                    instructions.extend_from_slice(&[0x48, 0xB8]);
                    instructions.extend_from_slice(&value.to_le_bytes());
                    
                    store(instructions, *destination, Register::Rax);
                },
                Instruction::ImmediateFloat { long, destination, value } => {
                    if *long {
                        // mov rax [value]
                        instructions.extend_from_slice(&[0x48, 0xB8]);
                        instructions.extend_from_slice(&value.to_le_bytes());
                    
                        store(instructions, *destination, Register::Rax);
                    }
                    else {
                        // mov eax [value]
                        instructions.push(0xB8);
                        instructions.extend_from_slice(&(*value as f32).to_le_bytes());

                        store(instructions, *destination, Register::Rax);
                    }
                },
                Instruction::Load { size, destination, address } => {
                    load(instructions, *address, 64, Register::Rax);

                    match size {
                        // mov rax, [rax]
                        64 => instructions.extend_from_slice(&[0x48, 0x8B, 0x00]),
                        32 => instructions.extend_from_slice(&[0x8B, 0x00]),
                        16 => instructions.extend_from_slice(&[0x66, 0x8B, 0x00]),
                        8 => instructions.extend_from_slice(&[0x8A, 0x00]),
                        _ => panic!(),
                    }
                    store(instructions, *destination, Register::Rax);
                },
                Instruction::Store { size, source, address } => {
                    load(instructions, *address, 64, Register::Rax);
                    load(instructions, *source, *size, Register::Rbx);

                    match size {
                        // mov [rax], rbx
                        64 => instructions.extend_from_slice(&[0x48, 0x89, 0x18]),
                        32 => instructions.extend_from_slice(&[0x89, 0x18]),
                        16 => instructions.extend_from_slice(&[0x66, 0x89, 0x18]),
                        8 => instructions.extend_from_slice(&[0x88, 0x18]),
                        _ => panic!(),
                    }
                },
                Instruction::BinaryOperation { size, operation, destination, left, right } => {
                    if operation.is_float() {
                        if *size == 32 {
                            load(instructions, *left, *size, Register::Xmm0);
                            load(instructions, *right, *size, Register::Xmm1);
                        }
                        else if *size == 64 {
                            load(instructions, *left, *size, Register::Xmm0d);
                            load(instructions, *right, *size, Register::Xmm1d);
                        }
                        else { panic!(); }
                    }
                    else {
                        load(instructions, *left, *size, Register::Rax);
                        load(instructions, *right, *size, Register::Rbx);
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
                        store(instructions, *destination, Register::Rax);
                    }
                    else if operation.is_float() {
                        if *size == 32 {
                            store(instructions, *destination, Register::Xmm0);
                        }
                        else if *size == 64 {
                            store(instructions, *destination, Register::Xmm0d);
                        }
                        else { panic!(); }
                    }
                    else { panic!(); }
                },
                Instruction::Call((name, parameters, ret)) => {
                    // push rbp
                    instructions.push(0x55);

                    for par in *parameters {
                        load(instructions, *par, 64, Register::Rax);
                        // push rax
                        instructions.push(0x50);
                    }
                    // lea rax, [rip + offset]
                    instructions.extend_from_slice(&[0x48, 0x8D, 0x05]);
                    function_calls.push((arena.alloc_slice(*name) as &[u8], instructions.len()));
                    instructions.extend_from_slice(&0u32.to_le_bytes());
                    // call rax
                    instructions.extend_from_slice(&[0xFF, 0xD0]);
                    
                    // add rsp [parameters*8]
                    instructions.extend_from_slice(&[0x48, 0x81, 0xC4]);
                    instructions.extend_from_slice(&(parameters.len() as u32 * 8).to_le_bytes());

                    // pop rbp
                    instructions.push(0x5D);

                    store(instructions, *ret, Register::Rax);
                },
                Instruction::Return { value } => {
                    if let Some(value) = value {
                        load(instructions, *value, 64, Register::Rax);
                    }
                    // sub rbp, [8*parameter_count+1]
                    instructions.extend_from_slice(&[0x48, 0x81, 0xED]);
                    instructions.extend_from_slice(&( (function.paramaters_count as u32 + 1) * 8).to_le_bytes());

                    instructions.extend_from_slice(&[
                        // move rsp, rbp
                        0x48, 0x89, 0xEC,
                        // ret
                        0xC3,
                    ]);
                },
                Instruction::Branch { condition, label } => {
                    load(instructions, *condition, 8, Register::Rax);
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
                            load(instructions, *source, *from_size, Register::Rax);
                            match (from_size, to_size) {
                                // movzx ax, al
                                (8, 16) => instructions.extend_from_slice(&[0x66, 0x0F, 0xB6, 0xC0]),
                                (8, 32) => instructions.extend_from_slice(&[0x0F, 0xB6, 0xC0]),
                                (8, 64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]),
                                (16, 32) => instructions.extend_from_slice(&[0x0F, 0xB7, 0xC0]),
                                (16, 64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0xC0]),
                                (32, 64) => instructions.extend_from_slice(&[0x89, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Rax);
                        },
                        ExtTruncType::SignExtend => {
                            load(instructions, *source, *from_size, Register::Rax);
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
                            store(instructions, *destination, Register::Rax);
                        },
                        ExtTruncType::Fextend => {
                            load(instructions, *source, *from_size, Register::Xmm0);
                            match (from_size, to_size) {
                                // cvtss2sd xmm0, xmm0
                                (32, 64) => instructions.extend_from_slice(&[0xF3, 0x0F, 0x5A, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Xmm0d);
                        },
                        ExtTruncType::Truncate => {
                            load(instructions, *source, *from_size, Register::Rax);
                            store(instructions, *destination, Register::Rax);
                        },
                        ExtTruncType::Ftruncate => {
                            load(instructions, *source, *from_size, Register::Xmm0d);
                            match (from_size, to_size) {
                                // cvtsd2ss xmm0, xmm0
                                (64, 32) => instructions.extend_from_slice(&[0xF2, 0x0F, 0x5A, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Xmm0);
                        },
                    }
                },
                Instruction::Convert { typ, source, from_size, destination, to_size } => {
                    match typ {
                        ConvertType::FloatToUnsigned | ConvertType::FloatToSigned => {
                            if  *from_size == 32 {
                                load(instructions, *source, *from_size, Register::Xmm0);
                                // cvttss2si rax, xmm0
                                instructions.extend_from_slice(&[0xF3, 0x48, 0x0F, 0x2C, 0xC0]);
                            }
                            else if *from_size == 64 {
                                load(instructions, *source, *from_size, Register::Xmm0d);
                                // cvttsd2si rax, xmm0
                                instructions.extend_from_slice(&[0xF2, 0x48, 0x0F, 0x2C, 0xC0]);
                            }
                            else { panic!(); }

                            store(instructions, *destination, Register::Rax);
                        },
                        ConvertType::SignedToFloat | ConvertType::UnsignedToFloat => {
                            load(instructions, *source, *from_size, Register::Rax);
                            if  *to_size == 32 {
                                // cvtsi2ss xmm0, rax
                                instructions.extend_from_slice(&[0xF3, 0x48, 0x0F, 0x2A, 0xC0]);
                                store(instructions, *destination, Register::Xmm0);
                            }
                            else if *from_size == 64 {
                                // cvtsi2sd xmm0, rax
                                instructions.extend_from_slice(&[0xF2, 0x48, 0x0F, 0x2A, 0xC0]);
                                store(instructions, *destination, Register::Xmm0d);
                            }
                            else { panic!(); }
                        },
                    }
                },
                Instruction::Move { source, destination } => {
                    load(instructions, *source, 64, Register::Rax);
                    store(instructions, *destination, Register::Rax);
                },
                Instruction::AddrOf { source, destination } => {
                    // lea rax, rbp + [-index]
                    instructions.extend_from_slice(&[0x48, 0x8D, 0x85]);
                    instructions.extend_from_slice(&( -(*source as i32) ).to_le_bytes());

                    store(instructions, *destination, Register::Rax);
                },
            }
        }
    }

    for (jmp, label) in jumps.array() {
        let offset = *labels.get(*label as usize).unwrap() as i32 - *jmp as i32 - 4;
        instructions[*jmp..*jmp+4].copy_from_slice(&offset.to_le_bytes());
    }
}
fn load(instructions: &mut Vec<u8>, source: u32, size: u8, register: Register) {
    match register {
        Register::Rax => {
            // mov rax, rbp + [-index]
            instructions.extend_from_slice(&[0x48, 0x8B, 0x85]);
        },
        Register::Rbx => {
            // mov rbx, rbp + [-index]
            instructions.extend_from_slice(&[0x48, 0x8B, 0x9D]);
        },
        Register::Xmm0 => {
            // movss xmm0, rbp + [-index] 
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x85]);               
        },
        Register::Xmm1 => {
            // movss xmm1, rbp + [-index] 
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x8D]);
        },
        Register::Xmm0d => {
            // movsd xmm0, rbp + [-index] 
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x85]);             
        },
        Register::Xmm1d => {
            // movsd xmm1, rbp + [-index] 
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x8D]);
        },
    }
    instructions.extend_from_slice(&(-(source as i32)).to_le_bytes());

    if register == Register::Rax {
        match size {
            // movzx rax, al
            8 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]),
            16 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0xC0]),
            32 => instructions.extend_from_slice(&[0x89, 0xC0]),
            64 => {},
            _ => panic!(),
        }
    } 
    if register == Register::Rbx {
        match size {
            // movzx rbx, bl
            8 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xDB]),
            16 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0xDB]),
            32 => instructions.extend_from_slice(&[0x89, 0xDB]),
            64 => {},
            _ => panic!(),
        }
    } 
}
fn store(instructions: &mut Vec<u8>, destination: u32, register: Register) {
    match register {
        Register::Rax => {
            // mov rbp + [-destination], rax
            instructions.extend_from_slice(&[0x48, 0x89, 0x85]);
        },
        Register::Xmm0 => {
            // movss rbp + [-destination], xmm0
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x11, 0x85]);
        },
        Register::Xmm0d => {
            // movsd rbp + [-destination], xmm0
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x11, 0x85]);
        },
        _ => panic!(),
    }
    instructions.extend_from_slice(&(-(destination as i32)).to_le_bytes());
}
#[derive(PartialEq)]
enum Register {
    Rax,
    Rbx,
    Xmm0,
    Xmm1,
    Xmm0d,
    Xmm1d,
}