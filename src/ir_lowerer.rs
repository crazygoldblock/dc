#![allow(unused)]

use crate::arena::{Arena, VecArena};
use crate::parser::{Expression, ASTfunction, Statement, Struct, Type, Union, ASTbinaryOperator, ASTunaryOperator};

// Check for missing returns and jumps to non existent blocks

// return function signatures
// alloca instruction for array inits

#[derive(Debug)]
pub struct IRfunction<'b> {
    pub name: &'b [u8],
    pub paramaters_count: u16,
    pub blocks: &'b [Block<'b>],
}
#[derive(Debug)]
pub struct Block<'b> {
    pub instructions: VecArena<'b, Instruction<'b>>,
}
#[derive(Debug)]
pub enum Instruction<'b> {
    NoOperation,
    Immediate{
        destination: u32,
        value: u64,
    },
    ImmediateFloat{
        typ: &'static str,
        destination: u32,
        value: f64,
    },
    Load{
        size: u8,
        destination: u32,
        address: u32,
    },
    Store{
        size: u8,
        source: u32,
        address: u32,
    },
    BinaryOperation{
        size: u8,
        operation: IRbinaryOperator,
        destination: u32,
        left: u32,
        right: u32,
    },
    Call(&'b ( 
        &'b [u8],
        &'b [u32],
        u32),
    ),
    Return{
        value: Option<u32>,
    },
    Branch{
        condition: u32,
        label: i32,
    },
    Jump{
        label: i32,
    },
    ExtendTruncate{
        typ: ExtTruncType,
        source: u32,
        from_size: u8,
        destination: u32,
        to_size: u8,
    },
    Convert{
        typ: ConvertType,
        source: u32,
        from_size: u8,
        destination: u32,
        to_size: u8,
    },
    Move{
        source: u32,
        destination: u32,
    },
    AddrOf{
        source: u32,
        destination: u32,
    },
    LoadGlobal{

    },
    StoreGlobal{

    }
}
#[derive(Debug)]
pub enum ConvertType {
    FloatToUnsigned,
    FloatToSigned,

    SignedToFloat,
    UnsignedToFloat,
}
#[derive(Debug)]
pub enum ExtTruncType {
    ZeroExtend,
    SignExtend,
    Fextend,

    Truncate,
    Ftruncate,
}
#[derive(Debug)]
pub enum IRbinaryOperator {
    Add,
    Sub,

    Mul,
    Udiv,
    Sdiv,
    Urem,
    Srem,

    And,
    Or,
    Xor,

    Shl,
    ShrZeroFill,
    ShrSignFill,

    Eq,
    NotEq,

    Ugreater,
    UgreaterEq,
    Uless,
    UlessEq,
    Sgreater,
    SgreaterEq,
    Sless,
    SlessEq,

    Feq,
    FnotEq,
    Fgreater,
    FgreaterEq,
    Fless,
    FlessEq,
    
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    Frem,
}
impl IRbinaryOperator {
    pub fn is_int(&self) -> bool {
        match self {
            IRbinaryOperator::Feq => false,
            IRbinaryOperator::FnotEq => false,
            IRbinaryOperator::Fgreater => false,
            IRbinaryOperator::FgreaterEq => false,
            IRbinaryOperator::Fless => false,
            IRbinaryOperator::FlessEq => false,
            IRbinaryOperator::Fadd => false,
            IRbinaryOperator::Fsub => false,
            IRbinaryOperator::Fmul => false,
            IRbinaryOperator::Fdiv => false,
            IRbinaryOperator::Frem => false,
            _ => true,
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            IRbinaryOperator::Feq => true,
            IRbinaryOperator::FnotEq => true,
            IRbinaryOperator::Fgreater => true,
            IRbinaryOperator::FgreaterEq => true,
            IRbinaryOperator::Fless => true,
            IRbinaryOperator::FlessEq => true,
            IRbinaryOperator::Fadd => true,
            IRbinaryOperator::Fsub => true,
            IRbinaryOperator::Fmul => true,
            IRbinaryOperator::Fdiv => true,
            IRbinaryOperator::Frem => true,
            _ => false,
        }
    }
    pub fn is_comparison(&self) -> bool {
        match self {
            IRbinaryOperator::Eq |
            IRbinaryOperator::NotEq |
            IRbinaryOperator::Ugreater |
            IRbinaryOperator::UgreaterEq |
            IRbinaryOperator::Uless |
            IRbinaryOperator::UlessEq |
            IRbinaryOperator::Sgreater |
            IRbinaryOperator::SgreaterEq |
            IRbinaryOperator::Sless |
            IRbinaryOperator::SlessEq |
            IRbinaryOperator::Feq |
            IRbinaryOperator::FnotEq |
            IRbinaryOperator::Fgreater |
            IRbinaryOperator::FgreaterEq |
            IRbinaryOperator::Fless |
            IRbinaryOperator::FlessEq => true,
            _ => false,
        }
    }
}
pub fn lower_function<'b>(
    arena: &'b Arena,
    function: &ASTfunction,
    functions: &[ASTfunction],
) -> Option<IRfunction<'b>> {
    let mut blocks = arena.vec_capacity(8);
    blocks.push(Block { instructions: arena.vec_capacity(16) });

    let mut variables = Vec::new();

    let mut stack_size = 0;

    let parameters = arena.alloc_array(RegisterType::Bool, function.parameters.len());

    for (i, p) in function.parameters.iter().enumerate() {
        let reg_type = type_to_register(&p.0, arena);
        parameters[i] = reg_type.clone();
        stack_size += 8;
        variables.push((p.1, stack_size, reg_type));
    }
    stack_size += 8;

    let return_type = if let Some(ret_t) = function.return_type.clone() {
        Some(type_to_register(&ret_t, arena))
    }
    else { None };

    for s in function.body {
        lower_statement(s, arena, &mut stack_size, &mut blocks, &mut variables, return_type.clone(), None, functions).unwrap();
    }

    return Some(IRfunction {
        name: arena.alloc_slice(function.name) as &[u8],
        paramaters_count: parameters.len() as u16,
        blocks: blocks.array(),
    });
}
#[derive(PartialEq, Clone, Debug)]
pub enum RegisterType<'b> {   
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    F32,
    F64,

    Bool,

    Ptr(VariableType<'b>),
}
#[derive(PartialEq, Clone, Debug)]
pub enum VariableType<'b> {
    Primitive(&'b RegisterType<'b>),
    Struct, //(Struct<'a, 'b>),
    Union, //(Union<'a, 'b>),
}
impl<'b> RegisterType<'b> {
    fn size(&self) -> u8 {
        match self {
            RegisterType::I8 => 8,
            RegisterType::I16 => 16,
            RegisterType::I32 => 32,
            RegisterType::I64 => 64,
            RegisterType::U8 => 8,
            RegisterType::U16 => 16,
            RegisterType::U32 => 32,
            RegisterType::U64 => 64,
            RegisterType::F32 => 32,
            RegisterType::F64 => 64,
            RegisterType::Bool => 8,
            RegisterType::Ptr(_) => 64,
        }
    }
    fn is_int(&self) -> bool {
        self.is_signed() || self.is_unsigned()
    }
    fn is_signed(&self) -> bool {
        match self {
            RegisterType::I8 |
            RegisterType::I16 |
            RegisterType::I32 |
            RegisterType::I64 => true,
            _=> false,
        }
    }
    fn is_unsigned(&self) -> bool {
        match self {
            RegisterType::U8 |
            RegisterType::U16 |
            RegisterType::U32 |
            RegisterType::U64 => true,
            _=> false,
        }
    }
    fn is_float(&self) -> bool {
        match self {
            RegisterType::F32 |
            RegisterType::F64 => true,
            _=> false,
        }
    }
    fn from_str(input: &[u8]) -> Option<RegisterType<'b>> {
        Some(match input {
            b"i8" => RegisterType::I8,
            b"i16" => RegisterType::I16,
            b"i32" => RegisterType::I32,
            b"i64" => RegisterType::I64,
            b"u8" => RegisterType::U8,
            b"u16" => RegisterType::U16,
            b"u32" => RegisterType::U32,
            b"u64" => RegisterType::U64,
            b"f32" => RegisterType::F32,
            b"f64" => RegisterType::F64,
            b"u32" => RegisterType::U32,
            b"bool" => RegisterType::Bool,
            _ => return None,
        })
    }
    fn is_ptr(&self) -> bool {
        matches!(self, RegisterType::Ptr(_))
    }
    fn ptr_element_size(&self) -> u64 {
        if let RegisterType::Ptr(ptr) = self {
            if let VariableType::Primitive(prim) = ptr {
                prim.size() as u64
            }
            else {
                panic!();
            }
        }
        else {
            panic!();
        }
    }
}
fn lower_expression<'a, 'b, 'c>(
    expression: &Expression, 
    arena: &'c Arena, 
    stack_size: &mut u32, 
    instructions: &mut VecArena<'c, Instruction<'c>>,
    variables: &mut Vec<(&'a [u8], u32, RegisterType<'c>)>,
    functions: &[ASTfunction],
) -> Option<(u32, RegisterType<'c>)> {

    match expression {
        Expression::Integer{ typ, value } => {
            let typ = RegisterType::from_str(typ.as_bytes()).unwrap();
            *stack_size += 8;
            instructions.push(Instruction::Immediate { destination: *stack_size, value: *value });
            return Some((*stack_size, typ));
        },
        Expression::Float{ typ, value} => {
            let typ_r = RegisterType::from_str(typ.as_bytes()).unwrap();
            *stack_size += 8;
            instructions.push(Instruction::ImmediateFloat { typ: *typ, destination: *stack_size, value: *value });
            return Some((*stack_size, typ_r));
        },
        Expression::Bool(b) => {
            *stack_size += 8;
            instructions.push(Instruction::Immediate { destination: *stack_size, value: *b as u64 });
            return Some((*stack_size, RegisterType::Bool));
        },
        Expression::Name(n) => {
            let pos = variables.iter().rev().find(|x| &x.0 == n).unwrap();
            *stack_size += 8;
            instructions.push(Instruction::Move { source: pos.1, destination: *stack_size });
            return Some((*stack_size, pos.2.clone()));
        },
        Expression::BinaryOperator { operator, left, right } => {
            let (left_offset, mut left_type) = lower_expression(left, arena, stack_size, instructions, variables, functions).unwrap();

            let (right_offset, mut right_type) = lower_expression(right, arena, stack_size, instructions, variables, functions).unwrap();

            let op = match operator {
                ASTbinaryOperator::Add => {
                    if left_type.is_ptr() && right_type.is_int() &&right_type.size() == 64 {
                        *stack_size += 8;
                        instructions.push(Instruction::Immediate { destination: *stack_size, value: left_type.ptr_element_size() / 8 });
                           
                        instructions.push(Instruction::BinaryOperation { 
                            size: 64, 
                            operation: IRbinaryOperator::Mul, 
                            destination: *stack_size, 
                            left: *stack_size, 
                            right: right_offset, 
                        });
                        (IRbinaryOperator::Add, left_type, 64)
                    }
                    else {
                        if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                            panic!();
                        }

                        let op = match left_type.is_float() {
                            true => IRbinaryOperator::Fadd,
                            false => IRbinaryOperator::Add,
                        };
                        (op, left_type.clone(), left_type.size())
                    }
                },
                ASTbinaryOperator::Sub => {
                    if left_type.is_ptr() && right_type.is_int() &&right_type.size() == 64 {
                        *stack_size += 8;
                        instructions.push(Instruction::Immediate { destination: *stack_size, value: left_type.ptr_element_size() / 8 });
                           
                        instructions.push(Instruction::BinaryOperation { 
                            size: 64, 
                            operation: IRbinaryOperator::Mul, 
                            destination: *stack_size, 
                            left: *stack_size, 
                            right: right_offset, 
                        });
                        (IRbinaryOperator::Sub, left_type, 64)
                    }
                    else {
                        if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                            panic!();
                        }

                        let op = match left_type.is_float() {
                            true => IRbinaryOperator::Fsub,
                            false => IRbinaryOperator::Sub,
                        };
                        let size = left_type.size();
                        (op, left_type, size)
                    }
                },
                ASTbinaryOperator::Mul => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match left_type.is_float() {
                        true => IRbinaryOperator::Fmul,
                        false => IRbinaryOperator::Mul,
                    };
                    let size = left_type.size();
                    (op, left_type, size)
                },
                ASTbinaryOperator::Div => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::Fdiv,
                        (false, true) => IRbinaryOperator::Sdiv,
                        (false, false) => IRbinaryOperator::Udiv,
                    };
                    let size = left_type.size();
                    (op, left_type, size)
                },
                ASTbinaryOperator::Rem => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::Frem,
                        (false, true) => IRbinaryOperator::Srem,
                        (false, false) => IRbinaryOperator::Urem,
                    };
                    let size = left_type.size();
                    (op, left_type, size)
                },
                ASTbinaryOperator::And => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) || left_type.is_float() {
                        panic!();
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::And, left_type, size)
                },
                ASTbinaryOperator::Xor => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) || left_type.is_float() {
                        panic!();
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Xor, left_type, size)
                },
                ASTbinaryOperator::Or => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) || left_type.is_float() {
                        panic!();
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Or, left_type, size)
                },
                ASTbinaryOperator::Shl => {
                    // TODO: out of bounds shift
                    if !right_type.is_unsigned() || !left_type.is_int() {
                        panic!();
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Shl, left_type, size)
                },
                ASTbinaryOperator::Shr => {
                    // TODO: out of bounds shift
                    if !right_type.is_unsigned() || !left_type.is_int() {
                        panic!();
                    }

                    let op = match left_type.is_signed() {
                        true => IRbinaryOperator::ShrSignFill,
                        false => IRbinaryOperator::ShrZeroFill,
                    };
                    let size = left_type.size();
                    (op, left_type, size)
                },
                ASTbinaryOperator::LogicalAnd => {
                    if left_type != RegisterType::Bool || right_type != RegisterType::Bool {
                        panic!();
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::And, left_type, size)
                },
                ASTbinaryOperator::LogicalOr => {
                    if left_type != RegisterType::Bool || right_type != RegisterType::Bool {
                        panic!();
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Or, left_type, size)
                },
                ASTbinaryOperator::Less => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::Fless,
                        (false, true) => IRbinaryOperator::Sless,
                        (false, false) => IRbinaryOperator::Uless,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::LessEq => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::FlessEq,
                        (false, true) => IRbinaryOperator::SlessEq,
                        (false, false) => IRbinaryOperator::UlessEq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::Greater => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::Fgreater,
                        (false, true) => IRbinaryOperator::Sgreater,
                        (false, false) => IRbinaryOperator::Ugreater,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::GreaterEq => {
                    if left_type != right_type || left_type == RegisterType::Bool || matches!(left_type, RegisterType::Ptr(_)) {
                        panic!();
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::FgreaterEq,
                        (false, true) => IRbinaryOperator::SgreaterEq,
                        (false, false) => IRbinaryOperator::UgreaterEq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::Eq => {
                    if left_type != right_type {
                        panic!();
                    }

                    let op = match left_type.is_float() {
                        true => IRbinaryOperator::Feq,
                        false => IRbinaryOperator::Eq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::NotEq => {
                    if left_type != right_type {
                        panic!();
                    }

                    let op = match left_type.is_float() {
                        true => IRbinaryOperator::FnotEq,
                        false => IRbinaryOperator::FnotEq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::Index => todo!(),
            };

            *stack_size += 8;

            instructions.push(Instruction::BinaryOperation { 
                size: op.2, 
                operation: op.0, 
                destination: *stack_size, 
                left: left_offset, 
                right: right_offset, 
            });
            return Some((*stack_size, op.1));
        },
        Expression::UnaryOperator { operator, expr } => {
            let (offset, mut typ) = lower_expression(expr, arena, stack_size, instructions, variables, functions).unwrap();

            let op = match operator {
                ASTunaryOperator::Sub => {
                    if !typ.is_int() {
                        panic!();
                    } 
                    let size = typ.size();
                    *stack_size += 8;
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: 0 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: IRbinaryOperator::Sub, 
                        destination: *stack_size, 
                        left: *stack_size, 
                        right: offset,
                    });
                    return Some((*stack_size, typ));
                },
                ASTunaryOperator::Not => {
                    if !typ.is_int() {
                        panic!();
                    } 
                    let size = typ.size();
                    *stack_size += 8;
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: 1u64.wrapping_shl(size as u32)-1 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: IRbinaryOperator::Xor, 
                        destination: *stack_size, 
                        left: offset, 
                        right: *stack_size,
                    });
                    return Some((*stack_size, typ));
                },
                ASTunaryOperator::Invert => {
                    if typ != RegisterType::Bool {
                        panic!();
                    } 
                    let size = typ.size();
                    *stack_size += 8;
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: 1 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: 8, 
                        operation: IRbinaryOperator::Xor, 
                        destination: *stack_size, 
                        left: offset, 
                        right: *stack_size,
                    });
                    return Some((*stack_size, typ));
                },
                ASTunaryOperator::Dereference => {
                    if let RegisterType::Ptr(typ) = typ {
                        if let VariableType::Primitive(prim) = typ {
                            *stack_size += 8;
                            instructions.push(Instruction::Load { size: (prim.ptr_element_size()/8) as u8, destination: *stack_size, address: offset });
                            return Some((*stack_size, prim.clone()));
                        }
                        else { panic!(); }
                    }
                    else { panic!(); }
                },
                ASTunaryOperator::Address => {
                    let typ = RegisterType::Ptr(VariableType::Primitive(arena.alloc(typ)));
                    *stack_size += 8;
                    instructions.push(Instruction::AddrOf { source: offset, destination: *stack_size });
                    return Some((*stack_size, typ));
                },
            };
        },
        Expression::Cast { expr, typ } => {
            // TODO: pointer casts

            let (offset, left_t) = lower_expression(expr, arena, stack_size, instructions, variables, functions).unwrap();

            let cast_type = type_to_register(typ, arena);
            let cast_size = cast_type.size();

            if left_t.is_ptr() {
                panic!();
            }

            if left_t == RegisterType::Bool && cast_type.is_float() {
                panic!();
            }

            if left_t == cast_type {
                panic!();
            }

            *stack_size += 8;

            if left_t.is_float() && cast_type.is_float() {
                instructions.push(Instruction::ExtendTruncate { 
                    typ: if left_t == RegisterType::F32 { ExtTruncType::Fextend } else { ExtTruncType::Ftruncate }, 
                    source: offset, 
                    from_size: left_t.size(), 
                    destination: *stack_size, 
                    to_size: cast_size 
                });
            }
            else if left_t.is_int() && cast_type.is_int() {
                if left_t.size() == cast_type.size() {
                    instructions.push(Instruction::Move { source: offset, destination: *stack_size });
                }
                else {
                    instructions.push(Instruction::ExtendTruncate { 
                        typ: if left_t.size() > cast_size { ExtTruncType::Truncate } else { 
                            if left_t.is_signed() { ExtTruncType::SignExtend } else { ExtTruncType::ZeroExtend } }, 
                        source: offset, 
                        from_size: left_t.size(), 
                        destination: *stack_size, 
                        to_size: cast_size 
                    });
                }
            }
            else if left_t.is_float() && cast_type == RegisterType::Bool {
                instructions.push(Instruction::ImmediateFloat { 
                    typ: if left_t.size() == 32 { "f32" } else { "f64" }, 
                    destination: *stack_size, 
                    value: 0.0 
                });
                instructions.push(Instruction::BinaryOperation { 
                    size: left_t.size(), 
                    operation: IRbinaryOperator::FnotEq, 
                    destination: *stack_size, 
                    left: offset, 
                    right: *stack_size 
                }); 
            }
            else if left_t.is_int() && cast_type.is_float() {
                instructions.push(Instruction::Convert { 
                    typ: if left_t.is_signed() { ConvertType::SignedToFloat } else { ConvertType::UnsignedToFloat }, 
                    source: offset, 
                    from_size: left_t.size(), 
                    destination: *stack_size, 
                    to_size: cast_size, 
                });
            }
            else if left_t.is_float() && cast_type.is_int() {
                instructions.push(Instruction::Convert { 
                    typ: if cast_type.is_signed() { ConvertType::FloatToSigned } else { ConvertType::FloatToUnsigned }, 
                    source: offset, 
                    from_size: left_t.size(), 
                    destination: *stack_size, 
                    to_size: cast_size, 
                });
            }
            else {
                panic!();
            }
            return Some((*stack_size, left_t));
        },
        Expression::Call { name, parameters } => {
            let register_paratemers = arena.alloc_array(0u32, parameters.len());
            let function = functions.iter().find(|x| x.name == *name).unwrap();

            for (i, par) in parameters.iter().enumerate() {
                let (offset, expression_type) = lower_expression(par, arena, stack_size, instructions, variables, functions).unwrap();
                register_paratemers[i] = offset;
            
                if type_to_register(&function.parameters[i].0, arena) != expression_type {
                    panic!();
                }  
            }

            *stack_size += 8;
            instructions.push(Instruction::Call ( 
                arena.alloc((arena.alloc_slice(name) as &[u8], 
                register_paratemers as &[u32],  
                *stack_size,
            ))));
            if function.return_type.is_none() {
                return None;
            }
            return Some((*stack_size, type_to_register(&function.return_type.clone().unwrap(), arena)));
        },
        Expression::Field { expr, field } => todo!(),
        Expression::ArrayInit(expressions) => {
            if expressions.len() == 0 {
                panic!();
            }
            
            let (exp_offset, exp_type) = lower_expression(&expressions[0], arena, stack_size, instructions, variables, functions).unwrap();
            let mut array_type = exp_type;

            *stack_size += (array_type.ptr_element_size()/8 * expressions.len() as u64) as u32;
            *stack_size = stack_size.next_multiple_of(8);
            let array_offset = *stack_size;

            *stack_size += 8;
            let ret_type = RegisterType::Ptr(VariableType::Primitive(arena.alloc(array_type.clone())));
            instructions.push(Instruction::AddrOf { source: array_offset, destination: *stack_size });
            let ptr_offset = *stack_size;

            *stack_size += 8;
            instructions.push(Instruction::Immediate { destination: *stack_size, value: array_type.ptr_element_size() });
            let element_size_offset = *stack_size;

            instructions.push(Instruction::Store { size: array_type.ptr_element_size() as u8, source: exp_offset, address: ptr_offset });
            
            for (i, e) in expressions[1..].iter().enumerate() {
                let (exp_offset, exp_type) = lower_expression(e, arena, stack_size, instructions, variables, functions).unwrap();

                if exp_type != array_type {
                    panic!();
                }
                
                instructions.push(Instruction::BinaryOperation { size: 64, operation: IRbinaryOperator::Add, destination: ptr_offset, left: ptr_offset, right: element_size_offset });
                instructions.push(Instruction::Store { size: array_type.ptr_element_size() as u8, source: exp_offset, address: ptr_offset });
            }
            return Some((ptr_offset, ret_type));
        },
        Expression::StructInit(items) => todo!(),
    }
    panic!();
}
fn lower_statement<'a, 'b, 'c>(
    statement: &Statement<'a, 'b>,
    arena: &'c Arena,
    stack_size: &mut u32, 
    blocks: &mut VecArena<'c, Block<'c>>,
    variables: &mut Vec<(&'a [u8], u32, RegisterType<'c>)>,
    return_type: Option<RegisterType<'c>>,
    last_loop_start: Option<i32>,
    functions: &[ASTfunction],
) -> Option<()> {
    match statement {
        Statement::Assignment { destination, value } => {
            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let value = lower_expression(value, arena, stack_size, instructions, variables, functions);

            if let Some(d) = destination {
                if let Expression::Name(n) = d {
                    let (_, dest_offset, dest_type) = variables.iter().find(|x| &x.0 == n).unwrap();

                    if *dest_type != value.clone().unwrap().1 {
                        panic!();
                    }

                    instructions.push(Instruction::Move { source: value.unwrap().0, destination: *dest_offset });
                }
            }
        },
        Statement::Return { value } => {
            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let value_reg = if let Some(v) = value {
                let (value_offset, value_type) = lower_expression(v, arena, stack_size, instructions, variables, functions).unwrap();
                if return_type.is_none() || return_type.unwrap() != value_type {
                    panic!();
                }
                Some(value_offset)
            }
            else {
                if return_type.is_some() {
                    panic!();
                } 
                None 
            };

            instructions.push(Instruction::Return { value: value_reg });
        },
        Statement::If { condition, body, else_body } => {
            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let (c_offset, mut c_type) = lower_expression(condition, arena, stack_size, instructions, variables, functions).unwrap();

            if c_type != RegisterType::Bool {
                panic!();
            }

            *stack_size += 8;

            instructions.push(Instruction::Immediate { destination: *stack_size, value: 1 });
            instructions.push(Instruction::BinaryOperation { 
                size: 8, 
                operation: IRbinaryOperator::Xor, 
                destination: *stack_size, 
                left: *stack_size, 
                right: c_offset, 
            });

            let condition_reg = *stack_size;
            let condition_index = blocks.len()-1;
            blocks.push(Block { instructions: arena.vec_capacity(16) });

            let var_len = variables.len();

            for s in *body {
                lower_statement(s, arena, stack_size, blocks, variables, return_type.clone(), last_loop_start, functions).unwrap();
            }

            variables.truncate(var_len);

            let next_index = blocks.len();

            blocks.get_mut(condition_index).unwrap().instructions.push(Instruction::Branch { 
                condition: condition_reg, 
                label: next_index as i32, 
            });

            if let Some(e) = else_body {
                let jump_index = blocks.len()-1;
                
                blocks.push(Block { instructions: arena.vec_capacity(16) });

                let var_len = variables.len();

                for s in *e {
                    lower_statement(s, arena, stack_size, blocks, variables, return_type.clone(), last_loop_start, functions).unwrap();
                }

                while variables.len() > var_len {
                    variables.pop();
                }

                let next_index = blocks.len();

                blocks.get_mut(jump_index).unwrap().instructions.push(Instruction::Jump {  
                    label: next_index as i32, 
                });
            }

            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::While { condition, body } => {
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let condition_block_index = blocks.len()-1;

            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let (c_offset, mut c_type) = lower_expression(condition, arena, stack_size, instructions, variables, functions).unwrap();

            if c_type != RegisterType::Bool {
                panic!();
            }

            *stack_size += 8;

            instructions.push(Instruction::Immediate { destination: *stack_size, value: 1 });
            instructions.push(Instruction::BinaryOperation { 
                size: 8, 
                operation: IRbinaryOperator::Xor, 
                destination: *stack_size, 
                left: *stack_size, 
                right: c_offset, 
            });

            let condition_reg = *stack_size;

            blocks.push(Block { instructions: arena.vec_capacity(16) });

            let var_len = variables.len();

            for s in *body {
                lower_statement(s, arena, stack_size, blocks, variables, return_type.clone(), Some(condition_block_index as i32), functions).unwrap();
            }

            while variables.len() > var_len {
                variables.pop();
            }

            blocks.last_mut().unwrap().instructions.push(Instruction::Jump { 
                label: condition_block_index as i32, 
            });

            let next_block_index = blocks.len();

            blocks.get_mut(condition_block_index).unwrap().instructions.push(Instruction::Branch { 
                condition: condition_reg, 
                label: next_block_index as i32, 
            });

            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::Break => todo!(),
        Statement::Continue => {
            if let Some(i) = last_loop_start {
                blocks.last_mut().unwrap().instructions.push(Instruction::Jump { label: i });
            }
            else { panic!(); }
        },
        Statement::Declaration { typ, name, value } => {
            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let Type::Name(var_type) = typ else { panic!(); };
            let reg_t = RegisterType::from_str(*var_type).unwrap();

            if variables.iter().rev().find(|x| &x.0 == name).is_some() {
                panic!();
            }

            if let Some(v) = value {
                let (value_offset, mut value_type) = lower_expression(v, arena, stack_size, instructions, variables, functions).unwrap();
                
                if value_type != reg_t {
                    panic!();
                }
                variables.push((name, value_offset, reg_t));
            }
            else {
                *stack_size += 8;
                variables.push((name, *stack_size, reg_t));
            }
        },
        Statement::For { declaration, condition, iteration, body } => {
            let var_len = variables.len();

            if let Some(d) = declaration {
                lower_statement(d, arena, stack_size, blocks, variables, return_type.clone(), last_loop_start, functions).unwrap();
            }
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let condition_block_index = blocks.len()-1;

            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let condition_reg = if let Some(cond) = condition {
                let (c_offset, mut c_type) = lower_expression(cond, arena, stack_size, instructions, variables, functions).unwrap();

                if c_type != RegisterType::Bool {
                    panic!();
                }

                *stack_size += 8;

                instructions.push(Instruction::Immediate { destination: *stack_size, value: 1 });
                instructions.push(Instruction::BinaryOperation { 
                    size: 8, 
                    operation: IRbinaryOperator::Xor, 
                    destination: *stack_size, 
                    left: *stack_size, 
                    right: c_offset, 
                });

                Some(*stack_size)
            }
            else { None };
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });

            for s in *body {
                lower_statement(s, arena, stack_size, blocks, variables, return_type.clone(), Some(condition_block_index as i32), functions).unwrap();
            }

            if let Some(ite) = iteration {
                lower_statement(ite, arena, stack_size, blocks, variables, return_type.clone(), Some(condition_block_index as i32), functions).unwrap();
            }

            while variables.len() > var_len {
                variables.pop();
            }

            blocks.last_mut().unwrap().instructions.push(Instruction::Jump { 
                label: condition_block_index as i32, 
            });

            let next_block_index = blocks.len();

            if let Some(cond_reg) = condition_reg {
                blocks.get_mut(condition_block_index).unwrap().instructions.push(Instruction::Branch { 
                    condition: cond_reg, 
                    label: next_block_index as i32, 
                });
            }
            else {
                blocks.get_mut(condition_block_index).unwrap().instructions.push(Instruction::Jump {  
                    label: next_block_index as i32, 
                });
            }
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::DoWhile { body, condition } => {
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let start_block_index = blocks.len()-1;

            let var_len = variables.len();

            for s in *body {
                lower_statement(s, arena, stack_size, blocks, variables, return_type.clone(), Some(start_block_index as i32), functions).unwrap();
            }

            while variables.len() > var_len {
                variables.pop();
            }

            let instructions = &mut blocks.last_mut().unwrap().instructions;
            let (c_offset, mut c_type) = lower_expression(condition, arena, stack_size, instructions, variables, functions).unwrap();

            if c_type != RegisterType::Bool {
                panic!();
            }

            let next_block_index = blocks.len();

            blocks.get_mut(start_block_index).unwrap().instructions.push(Instruction::Branch { 
                condition: c_offset, 
                label: next_block_index as i32, 
            });

            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::Switch { value, arms, default } => todo!(),
    }
    return Some(());
}
fn type_to_register<'b>(typ: &Type, arena: &'b Arena) -> RegisterType<'b> {
    match typ {
        Type::Name(name) => RegisterType::from_str(*name).unwrap(),
        Type::Pointer(ptr) => RegisterType::Ptr(VariableType::Primitive(arena.alloc(type_to_register(ptr, arena)))),
    }
}