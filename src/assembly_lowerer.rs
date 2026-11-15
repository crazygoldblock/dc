use crate::arena::Arena;
use crate::ir_lowerer::IRfunction;
use crate::ir_lowerer::Instruction;
use crate::ir_lowerer::IRbinaryOperator;
use crate::ir_lowerer::ExtTruncType;
use crate::ir_lowerer::ConvertType;
use crate::ir_lowerer::Size;

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
                Instruction::Immediate { destination, value, size } => {
                    // mov rax [value]
                    instructions.extend_from_slice(&[0x48, 0xB8]);
                    instructions.extend_from_slice(&value.to_le_bytes());
                    
                    store(instructions, *destination, Register::Rax, *size);
                },
                Instruction::ImmediateFloat { long, destination, value } => {
                    if *long {
                        // mov rax [value]
                        instructions.extend_from_slice(&[0x48, 0xB8]);
                        instructions.extend_from_slice(&value.to_le_bytes());
                    
                        store(instructions, *destination, Register::Rax, Size::S64);
                    }
                    else {
                        // mov eax [value]
                        instructions.push(0xB8);
                        instructions.extend_from_slice(&(*value as f32).to_le_bytes());

                        store(instructions, *destination, Register::Rax, Size::S32);
                    }
                },
                Instruction::Load { size, destination, address } => {
                    load(instructions, *address, Register::Rax, Size::S64);

                    match size {
                        // mov rax, [rax]
                        Size::S64 => instructions.extend_from_slice(&[0x48, 0x8B, 0x00]),
                        Size::S32 => instructions.extend_from_slice(&[0x8B, 0x00]),
                        Size::S16 => instructions.extend_from_slice(&[0x66, 0x8B, 0x00]),
                        Size::S8 => instructions.extend_from_slice(&[0x8A, 0x00]),
                    }
                    store(instructions, *destination, Register::Rax, *size);
                },
                Instruction::Store { size, source, address } => {
                    load(instructions, *address, Register::Rax, Size::S64);
                    load(instructions, *source, Register::Rbx, *size);

                    match size {
                        // mov [rax], rbx
                        Size::S64 => instructions.extend_from_slice(&[0x48, 0x89, 0x18]),
                        Size::S32 => instructions.extend_from_slice(&[0x89, 0x18]),
                        Size::S16 => instructions.extend_from_slice(&[0x66, 0x89, 0x18]),
                        Size::S8 => instructions.extend_from_slice(&[0x88, 0x18]),
                    }
                },
                Instruction::BinaryOperation { size, operation, destination, left, right } => {
                    if operation.is_float() {
                        if *size == Size::S32 {
                            load(instructions, *left, Register::Xmm0, *size);
                            load(instructions, *right, Register::Xmm1, *size);
                        }
                        else if *size == Size::S64 {
                            load(instructions, *left, Register::Xmm0d, *size);
                            load(instructions, *right, Register::Xmm1d, *size);
                        }
                        else { panic!(); }
                    }
                    else {
                        load(instructions, *left, Register::Rax, *size);
                        load(instructions, *right, Register::Rbx, *size);
                    }

                    match operation {
                        IRbinaryOperator::Add => {
                            // add rax, rbx
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x01, 0xD8]),
                                Size::S32 => instructions.extend_from_slice(&[0x01, 0xD8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x01, 0xD8]),
                                Size::S8 => instructions.extend_from_slice(&[0x00, 0xD8]),
                            }
                        },
                        IRbinaryOperator::Sub => {
                            // sub rax, rbx
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x29, 0xD8]),
                                Size::S32 => instructions.extend_from_slice(&[0x29, 0xD8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x29, 0xD8]),
                                Size::S8 => instructions.extend_from_slice(&[0x28, 0xD8]),
                            }
                        },
                        IRbinaryOperator::Mul => {
                            // imul rax, rbx
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x0F, 0xAF, 0xC3]),
                                Size::S32 => instructions.extend_from_slice(&[0x0F, 0xAF, 0xC3]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x0F, 0xAF, 0xC3]),
                                Size::S8 => instructions.extend_from_slice(&[0xF6, 0xEB]),
                            }
                        },
                        IRbinaryOperator::Udiv => {
                            match size {
                                Size::S64 => { 
                                    instructions.extend_from_slice(&[
                                        // xor rdx, rdx
                                        0x48, 0x31, 0xD2,
                                        // div rbx
                                        0x48, 0xF7, 0xF3,
                                    ]);
                                },
                                Size::S32 => {
                                    instructions.extend_from_slice(&[
                                        // xor edx, edx
                                        0x31, 0xD2,
                                        // div ebx
                                        0xF7, 0xF3,
                                    ]);
                                },
                                Size::S16 => {
                                    instructions.extend_from_slice(&[
                                        // xor dx, dx
                                        0x66, 0x31, 0xD2,
                                        // div bx
                                        0x66, 0xF7, 0xF3,
                                    ]);
                                },
                                Size::S8 => {
                                    instructions.extend_from_slice(&[
                                        // xor ah, ah
                                        0x30, 0xE4,
                                        // div bl
                                        0xF6, 0xF3,
                                    ]);
                                },
                            }
                        },
                        IRbinaryOperator::Sdiv => {
                            match size {
                                Size::S64 => { 
                                    instructions.extend_from_slice(&[
                                        // cqo
                                        0x48, 0x99,
                                        // idiv rbx
                                        0x48, 0xF7, 0xFB,
                                    ]);
                                },
                                Size::S32 => {
                                    instructions.extend_from_slice(&[
                                        // cdq
                                        0x99,
                                        // idiv ebx
                                        0xF7, 0xFB,
                                    ]);
                                },
                                Size::S16 => {
                                    instructions.extend_from_slice(&[
                                        // cwd
                                        0x66, 0x99,
                                        // idiv bx
                                        0x66, 0xF7, 0xFB,
                                    ]);
                                },
                                Size::S8 => {
                                    instructions.extend_from_slice(&[
                                        // cbw
                                        0x66, 0x98,
                                        // idiv bl
                                        0xF6, 0xFB,
                                    ]);
                                },
                            }
                        },
                        IRbinaryOperator::Urem => {
                            match size {
                                Size::S64 => { 
                                    instructions.extend_from_slice(&[
                                        // xor rdx, rdx
                                        0x48, 0x31, 0xD2,
                                        // div rbx
                                        0x48, 0xF7, 0xF3,
                                        // mov rax, rdx
                                        0x48, 0x89, 0xD0,
                                    ]);
                                },
                                Size::S32 => {
                                    instructions.extend_from_slice(&[
                                        // xor edx, edx
                                        0x31, 0xD2,
                                        // div ebx
                                        0xF7, 0xF3,
                                        // mov eax, edx
                                        0x89, 0xD0,
                                    ]);
                                },
                                Size::S16 => {
                                    instructions.extend_from_slice(&[
                                        // xor dx, dx
                                        0x66, 0x31, 0xD2,
                                        // div bx
                                        0x66, 0xF7, 0xF3,
                                        // mov ax, dx
                                        0x66, 0x89, 0xD0,
                                    ]);
                                },
                                Size::S8 => {
                                    instructions.extend_from_slice(&[
                                        // xor ah, ah
                                        0x30, 0xE4,
                                        // div bl
                                        0xF6, 0xF3,
                                        // mov al, dl
                                        0x88, 0xD0,
                                    ]);
                                },
                            }
                        },
                        IRbinaryOperator::Srem => {
                            match size {
                                Size::S64 => { 
                                    instructions.extend_from_slice(&[
                                        // cqo
                                        0x48, 0x99,
                                        // idiv rbx
                                        0x48, 0xF7, 0xFB,
                                        // mov rax, rdx
                                        0x48, 0x89, 0xD0,
                                    ]);
                                },
                                Size::S32 => {
                                    instructions.extend_from_slice(&[
                                        // cdq
                                        0x99,
                                        // idiv ebx
                                        0xF7, 0xFB,
                                        // mov eax, edx
                                        0x89, 0xD0,
                                    ]);
                                },
                                Size::S16 => {
                                    instructions.extend_from_slice(&[
                                        // cwd
                                        0x66, 0x99,
                                        // idiv bx
                                        0x66, 0xF7, 0xFB,
                                        // mov ax, dx
                                        0x66, 0x89, 0xD0,
                                    ]);
                                },
                                Size::S8 => {
                                    instructions.extend_from_slice(&[
                                        // cbw
                                        0x66, 0x98,
                                        // idiv bl
                                        0xF6, 0xFB,
                                        // mov al. dl
                                        0x88, 0xD0,
                                    ]);
                                },
                            }
                        },
                        IRbinaryOperator::And => {
                            // and rax, rbx
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x21, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x21, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x21, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x20, 0xd8]),
                            }
                        },
                        IRbinaryOperator::Or => {
                            // or rax, rbx
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x09, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x09, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x09, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x08, 0xd8]),
                            }
                        },
                        IRbinaryOperator::Xor => {
                            // xor rax, rbx
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x31, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x31, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x31, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x30, 0xd8]),
                            }
                        },
                        IRbinaryOperator::Shl => {
                            // mov rcx, rbx
                            instructions.extend_from_slice(&[0x48, 0x89, 0xd9]);
                            // shl rax cl
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0xD3, 0xE0]),
                                Size::S32 => instructions.extend_from_slice(&[0xD3, 0xE0]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0xD3, 0xE0]),
                                Size::S8 => instructions.extend_from_slice(&[0xD2, 0xE0]),
                            }
                        },
                        IRbinaryOperator::ShrZeroFill => {
                            // mov rcx, rbx
                            instructions.extend_from_slice(&[0x48, 0x89, 0xd9]);
                            // shr rax cl
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0xD3, 0xE8]),
                                Size::S32 => instructions.extend_from_slice(&[0xD3, 0xE8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0xD3, 0xE8]),
                                Size::S8 => instructions.extend_from_slice(&[0xD2, 0xE8]),
                            }
                        },
                        IRbinaryOperator::ShrSignFill => {
                            // mov rcx, rbx
                            instructions.extend_from_slice(&[0x48, 0x89, 0xd9]);
                            // sar rax cl
                            match size {
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0xD3, 0xF8]),
                                Size::S32 => instructions.extend_from_slice(&[0xD3, 0xF8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0xD3, 0xF8]),
                                Size::S8 => instructions.extend_from_slice(&[0xD2, 0xF8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
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
                                Size::S64 => instructions.extend_from_slice(&[0x48, 0x39, 0xd8]),
                                Size::S32 => instructions.extend_from_slice(&[0x39, 0xd8]),
                                Size::S16 => instructions.extend_from_slice(&[0x66, 0x39, 0xd8]),
                                Size::S8 => instructions.extend_from_slice(&[0x38, 0xd8]),
                            }
                            instructions.extend_from_slice(&[
                                // setle al        
                                0x0F, 0x9E, 0xC0,
                                // movzx rax, al 
                                0x48, 0x0F, 0xB6, 0xC0,
                            ]);
                        },
                        IRbinaryOperator::Feq => {
                            if *size == Size::S32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == Size::S64 {
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
                            if *size == Size::S32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == Size::S64 {
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
                            if *size == Size::S32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == Size::S64 {
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
                            if *size == Size::S32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == Size::S64 {
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
                            if *size == Size::S32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == Size::S64 {
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
                            if *size == Size::S32 {
                                // ucomiss xmm0, xmm1
                                instructions.extend_from_slice(&[0x0F, 0x2E, 0xC1]);
                            }
                            else if *size == Size::S64 {
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
                            if *size == Size::S32 {
                                // addss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x58, 0xC1]);
                            }
                            else if *size == Size::S64 {
                                // addsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x58, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Fsub => {
                            if *size == Size::S32 {
                                // subss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x5C, 0xC1]);
                            }
                            else if *size == Size::S64 {
                                // subsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x5C, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Fmul => {
                            if *size == Size::S32 {
                                // mulss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x59, 0xC1]);
                            }
                            else if *size == Size::S64 {
                                // mulsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x59, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Fdiv => {
                            if *size == Size::S32 {
                                // divss xmm0, xmm1
                                instructions.extend_from_slice(&[0xF3, 0x0F, 0x5E, 0xC1]);
                            }
                            else if *size == Size::S64 {
                                // divsd xmm0, xmm1
                                instructions.extend_from_slice(&[0xF2, 0x0F, 0x5E, 0xC1]);
                            }
                            else { panic!(); }
                        },
                        IRbinaryOperator::Frem => todo!(),
                    }
                    if operation.is_int() || (operation.is_float() && operation.is_comparison()) {    
                        store(instructions, *destination, Register::Rax, *size);
                    }
                    else if operation.is_float() {
                        if *size == Size::S32 {
                            store(instructions, *destination, Register::Xmm0, *size);
                        }
                        else if *size == Size::S64 {
                            store(instructions, *destination, Register::Xmm0d, *size);
                        }
                        else { panic!(); }
                    }
                    else { panic!(); }
                },
                Instruction::Call((name, parameters, ret, ret_size)) => {
                    // push rbp
                    instructions.push(0x55);

                    for par in *parameters {
                        load(instructions, *par, Register::Rax, Size::S64);
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

                    store(instructions, *ret, Register::Rax, *ret_size);
                },
                Instruction::Return { value } => {
                    if let Some(value) = value {
                        load(instructions, value.0, Register::Rax, value.1);
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
                    load(instructions, *condition, Register::Rax, Size::S8);
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
                            load(instructions, *source, Register::Rax, *from_size);
                            match (from_size, to_size) {
                                // movzx ax, al
                                (Size::S8, Size::S16) => instructions.extend_from_slice(&[0x66, 0x0F, 0xB6, 0xC0]),
                                (Size::S8, Size::S32) => instructions.extend_from_slice(&[0x0F, 0xB6, 0xC0]),
                                (Size::S8, Size::S64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]),
                                (Size::S16, Size::S32) => instructions.extend_from_slice(&[0x0F, 0xB7, 0xC0]),
                                (Size::S16, Size::S64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0xC0]),
                                (Size::S32, Size::S64) => instructions.extend_from_slice(&[0x89, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Rax, *to_size);
                        },
                        ExtTruncType::SignExtend => {
                            load(instructions, *source, Register::Rax, *from_size);
                            match (from_size, to_size) {
                                // movsx ax, al
                                (Size::S8, Size::S16) => instructions.extend_from_slice(&[0x66, 0x0F, 0xBE, 0xC0]),
                                (Size::S8, Size::S32) => instructions.extend_from_slice(&[0x0F, 0xBE, 0xC0]),
                                (Size::S8, Size::S64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xBE, 0xC0]),
                                (Size::S16, Size::S32) => instructions.extend_from_slice(&[0x0F, 0xBF, 0xC0]),
                                (Size::S16, Size::S64) => instructions.extend_from_slice(&[0x48, 0x0F, 0xBF, 0xC0]),
                                (Size::S32, Size::S64) => instructions.extend_from_slice(&[0x48, 0x63, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Rax, *to_size);
                        },
                        ExtTruncType::Fextend => {
                            load(instructions, *source, Register::Xmm0, *from_size);
                            match (from_size, to_size) {
                                // cvtss2sd xmm0, xmm0
                                (Size::S32, Size::S64) => instructions.extend_from_slice(&[0xF3, 0x0F, 0x5A, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Xmm0d, *to_size);
                        },
                        ExtTruncType::Truncate => {
                            load(instructions, *source, Register::Rax, *from_size);
                            store(instructions, *destination, Register::Rax, *to_size);
                        },
                        ExtTruncType::Ftruncate => {
                            load(instructions, *source, Register::Xmm0d, *from_size);
                            match (from_size, to_size) {
                                // cvtsd2ss xmm0, xmm0
                                (Size::S64, Size::S32) => instructions.extend_from_slice(&[0xF2, 0x0F, 0x5A, 0xC0]),
                                _ => panic!(),
                            }
                            store(instructions, *destination, Register::Xmm0, *to_size);
                        },
                    }
                },
                Instruction::Convert { typ, source, from_size, destination, to_size } => {
                    match typ {
                        ConvertType::FloatToUnsigned | ConvertType::FloatToSigned => {
                            if  *from_size == Size::S32 {
                                load(instructions, *source, Register::Xmm0, *from_size);
                                // cvttss2si rax, xmm0
                                instructions.extend_from_slice(&[0xF3, 0x48, 0x0F, 0x2C, 0xC0]);
                            }
                            else if *from_size == Size::S64 {
                                load(instructions, *source, Register::Xmm0d, *from_size);
                                // cvttsd2si rax, xmm0
                                instructions.extend_from_slice(&[0xF2, 0x48, 0x0F, 0x2C, 0xC0]);
                            }
                            else { panic!(); }

                            store(instructions, *destination, Register::Rax, *to_size);
                        },
                        ConvertType::SignedToFloat | ConvertType::UnsignedToFloat => {
                            load(instructions, *source, Register::Rax, *from_size);
                            if  *to_size == Size::S32 {
                                // cvtsi2ss xmm0, rax
                                instructions.extend_from_slice(&[0xF3, 0x48, 0x0F, 0x2A, 0xC0]);
                                store(instructions, *destination, Register::Xmm0, *to_size);
                            }
                            else if *from_size == Size::S64 {
                                // cvtsi2sd xmm0, rax
                                instructions.extend_from_slice(&[0xF2, 0x48, 0x0F, 0x2A, 0xC0]);
                                store(instructions, *destination, Register::Xmm0d, *to_size);
                            }
                            else { panic!(); }
                        },
                    }
                },
                Instruction::Move { source, destination, size } => {
                    load(instructions, *source, Register::Rax, *size);
                    store(instructions, *destination, Register::Rax, *size);
                },
                Instruction::AddrOf { source, destination } => {
                    // lea rax, rbp + [-index]
                    instructions.extend_from_slice(&[0x48, 0x8D, 0x85]);
                    instructions.extend_from_slice(&( -(*source as i32) ).to_le_bytes());

                    store(instructions, *destination, Register::Rax, Size::S64);
                },
            }
            if instructions.ends_with(&[0xff, 0xff, 0xff, 0x31]) {
                println!("{:?}", ins);
                panic!();
            }
        }
    }

    for (jmp, label) in jumps.array() {
        let offset = *labels.get(*label as usize).unwrap() as i32 - *jmp as i32 - 4;
        instructions[*jmp..*jmp+4].copy_from_slice(&offset.to_le_bytes());
    }
}
fn load(instructions: &mut Vec<u8>, source: u32, register: Register, size: Size) {
    match register {
        Register::Rax => {
            // movzx rax, byte rbp + [-index]
            match size {
                Size::S8 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0x85]),
                Size::S16 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0x85]),
                Size::S32 => instructions.extend_from_slice(&[0x8B, 0x85]),
                Size::S64 => instructions.extend_from_slice(&[0x48, 0x8B, 0x85]),
            }
        },
        Register::Rbx => {
            // mov rbx, rbp + [-index]
            match size {
                Size::S8 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB6, 0x9D]),
                Size::S16 => instructions.extend_from_slice(&[0x48, 0x0F, 0xB7, 0x9D]),
                Size::S32 => instructions.extend_from_slice(&[0x8B, 0x9D]),
                Size::S64 => instructions.extend_from_slice(&[0x48, 0x8B, 0x9D]),
            }
        },
        Register::Xmm0 => {
            if size != Size::S32 {
                panic!()
            }
            // movss xmm0, rbp + [-index] 
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x85]);               
        },
        Register::Xmm1 => {
            if size != Size::S32 {
                panic!()
            }
            // movss xmm1, rbp + [-index] 
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x10, 0x8D]);
        },
        Register::Xmm0d => {
            if size != Size::S64 {
                panic!()
            }
            // movsd xmm0, rbp + [-index] 
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x85]);             
        },
        Register::Xmm1d => {
            if size != Size::S64 {
                panic!()
            }
            // movsd xmm1, rbp + [-index] 
            instructions.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x8D]);
        },
    }
    instructions.extend_from_slice(&(-(source as i32)).to_le_bytes());
}
fn store(instructions: &mut Vec<u8>, destination: u32, register: Register, size: Size) {
    match register {
        Register::Rax => {
            // mov rbp + [-destination], rax
            match size {
                Size::S8 => instructions.extend_from_slice(&[0x88, 0x85]),
                Size::S16 => instructions.extend_from_slice(&[0x66, 0x89, 0x85]),
                Size::S32 => instructions.extend_from_slice(&[0x89, 0x85]),
                Size::S64 => instructions.extend_from_slice(&[0x48, 0x89, 0x85]),
            }
        },
        Register::Xmm0 => {
            if size != Size::S32 {
                panic!();
            }
            // movss rbp + [-destination], xmm0
            instructions.extend_from_slice(&[0xF3, 0x0F, 0x11, 0x85]);
        },
        Register::Xmm0d => {
            if size != Size::S64 {
                panic!();
            }
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