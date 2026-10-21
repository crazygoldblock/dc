#![allow(unused)]

use std::collections::HashMap;

use crate::arena::{Arena, VecArena};
use crate::parser::{Expression, ASTfunction, Statement, Struct, Type, Union, ASTbinaryOperator, ASTunaryOperator};

#[derive(Debug)]
pub struct IRfunction<'a, 'b> {
    name: &'a [u8],
    paramaters: &'b [RegisterType<'a, 'b>],
    return_type: Option<RegisterType<'a, 'b>>,
    blocks: &'b [Block<'a, 'b>],
}
#[derive(Debug)]
pub struct Block<'a, 'b> {
    instructions: VecArena<'b, Instruction<'a, 'b>>,
}
#[derive(Debug)]
pub enum Instruction<'a, 'b> {
    NoOperation,
    Immediate{
        destination: u16,
        value: i64,
    },
    ImmediateFloat{
        destination: u16,
        value: f64,
    },
    Load{
        size: u8,
        destination: u16,
        address: u16,
    },
    Store{
        size: u8,
        source: u16,
        address: u16,
    },
    BinaryOperation{
        size: u8,
        operation: IRbinaryOperator,
        destination: u16,
        left: u16,
        right: u16,
    },
    Call(&'b ( 
        &'a [u8],
        &'b [u16],
        Option<u16>),
    ),
    Return{
        value: Option<u16>,
    },
    Branch{
        condition: u16,
        label: i32,
    },
    Jump{
        label: i32,
    },
    ExtendTruncate{
        typ: ExtTruncType,
        source: u16,
        from_size: u8,
        destination: u16,
        to_size: u8,
    },
    Convert{
        typ: ConvertType,
        source: u16,
        from_size: u8,
        destination: u16,
        to_size: u8,
    },
    Move{
        source: u16,
        destination: u16,
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
#[derive(PartialEq, Clone, Debug)]
pub enum VariableType<'a, 'b> {
    Primitive(&'a str),
    Struct(Struct<'a, 'b>),
    Union(Union<'a, 'b>),
}
pub fn lower_function<'a, 'b>(
    arena: &'b Arena, 
    function: &ASTfunction<'a, 'b>, 
    functions: &HashMap<&'a [u8], ASTfunction<'a, 'b>>,
) -> Option<IRfunction<'a, 'b>> {
    let mut registers = Vec::new();
    let mut blocks = arena.vec_capacity(8);
    blocks.push(Block { instructions: arena.vec_capacity(16) });
    let mut variables = Vec::new();

    let parameters = arena.alloc_array(RegisterType::Bool, function.parameters.len());

    for (i, p) in function.parameters.iter().enumerate() {
        let reg_type = if let Type::Name(p_typ) = p.0 {
            RegisterType::from_str(p_typ)?
        }
        else { return None; };
        parameters[i] = reg_type.clone();
        registers.push(reg_type);
        variables.push((p.1, i as u16));
    }

    let return_type = if let Some(ret_t) = function.return_type.clone() {
        if let Type::Name(n) = ret_t {
            Some(RegisterType::from_str(n)?) 
        }
        else { None }
    }
    else { None };

    for s in function.body {
        lower_statement(s, arena, &mut registers, &mut blocks, &mut variables, functions, return_type.clone(), None)?;
    }

    return Some(IRfunction {
        name: function.name,
        paramaters: parameters,
        return_type: return_type,
        blocks: blocks.array(),
    });
}
#[derive(PartialEq, Clone, Debug)]
enum RegisterType<'a, 'b> {
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

    Ptr(VariableType<'a, 'b>),
}
impl<'a, 'b> RegisterType<'a, 'b> {
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
    fn from_str(input: &'a [u8]) -> Option<RegisterType<'a, 'b>> {
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
}
fn lower_expression<'a, 'b>(
    expression: &Expression<'a, 'b>, 
    arena: &'b Arena, 
    registers: &mut Vec<RegisterType<'a, 'b>>, 
    instructions: &mut VecArena<'b, Instruction<'a, 'b>>,
    variables: &mut Vec<(&'a [u8], u16)>,
    functions: &HashMap<&'a [u8], ASTfunction<'a, 'b>>,
) -> Option<u16> {

    match expression {
        Expression::Integer(i) => {
            registers.push(RegisterType::I64);
            instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: *i });
            return Some(registers.len() as u16 - 1);
        },
        Expression::Float(f) => {
            registers.push(RegisterType::F64);
            instructions.push(Instruction::ImmediateFloat { destination: registers.len() as u16 - 1, value: *f });
            return Some(registers.len() as u16 - 1);
        },
        Expression::Bool(b) => {
            registers.push(RegisterType::Bool);
            instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: *b as i64 });
            return Some(registers.len() as u16 - 1);
        },
        Expression::Name(n) => {
            let pos = variables.iter().find(|x| &x.0 == n)?;
            return Some(pos.1);
        },
        Expression::BinaryOperator { operator, left, right } => {
            let left_reg = lower_expression(left, arena, registers, instructions, variables, functions)?;

            let right_reg = lower_expression(right, arena, registers, instructions, variables, functions)?;

            let left_t = registers[left_reg as usize].clone();
            let right_t = registers[right_reg as usize].clone();

            let op = match operator {
                ASTbinaryOperator::Add => {
                    // TODO: pointer = Add(pointer, int)
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => IRbinaryOperator::Fadd,
                        false => IRbinaryOperator::Add,
                    };
                    (op, left_t.clone(), left_t.size())
                },
                ASTbinaryOperator::Sub => {
                    // TODO: pointer = Sub(pointer, int)
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => IRbinaryOperator::Fsub,
                        false => IRbinaryOperator::Sub,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ASTbinaryOperator::Mul => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => IRbinaryOperator::Fmul,
                        false => IRbinaryOperator::Mul,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ASTbinaryOperator::Div => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => IRbinaryOperator::Fdiv,
                        (false, true) => IRbinaryOperator::Sdiv,
                        (false, false) => IRbinaryOperator::Udiv,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ASTbinaryOperator::Rem => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => IRbinaryOperator::Frem,
                        (false, true) => IRbinaryOperator::Srem,
                        (false, false) => IRbinaryOperator::Urem,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ASTbinaryOperator::And => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) || left_t.is_float() {
                        return None;
                    }
                    let size = left_t.size();
                    (IRbinaryOperator::And, left_t, size)
                },
                ASTbinaryOperator::Xor => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) || left_t.is_float() {
                        return None;
                    }
                    let size = left_t.size();
                    (IRbinaryOperator::Xor, left_t, size)
                },
                ASTbinaryOperator::Or => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) || left_t.is_float() {
                        return None;
                    }
                    let size = left_t.size();
                    (IRbinaryOperator::Or, left_t, size)
                },
                ASTbinaryOperator::Shl => {
                    // TODO: out of bounds shift
                    if (!right_t.is_signed() && !right_t.is_unsigned()) || (!left_t.is_signed() && !left_t.is_unsigned()) {
                        return None;
                    }
                    let size = left_t.size();
                    (IRbinaryOperator::Shl, left_t, size)
                },
                ASTbinaryOperator::Shr => {
                    // TODO: out of bounds shift
                    if (!right_t.is_signed() && !right_t.is_unsigned()) || (!left_t.is_signed() && !left_t.is_unsigned()) {
                        return None;
                    }

                    let op = match left_t.is_signed() {
                        true => IRbinaryOperator::ShrSignFill,
                        false => IRbinaryOperator::ShrZeroFill,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ASTbinaryOperator::LogicalAnd => {
                    if left_t != RegisterType::Bool || right_t != RegisterType::Bool {
                        return None;
                    }
                    let size = left_t.size();
                    (IRbinaryOperator::And, left_t, size)
                },
                ASTbinaryOperator::LogicalOr => {
                    if left_t != RegisterType::Bool || right_t != RegisterType::Bool {
                        return None;
                    }
                    let size = left_t.size();
                    (IRbinaryOperator::Or, left_t, size)
                },
                ASTbinaryOperator::Less => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => IRbinaryOperator::Fless,
                        (false, true) => IRbinaryOperator::Sless,
                        (false, false) => IRbinaryOperator::Uless,
                    };
                    (op, RegisterType::Bool, left_t.size())
                },
                ASTbinaryOperator::LessEq => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => IRbinaryOperator::FlessEq,
                        (false, true) => IRbinaryOperator::SlessEq,
                        (false, false) => IRbinaryOperator::UlessEq,
                    };
                    (op, RegisterType::Bool, left_t.size())
                },
                ASTbinaryOperator::Greater => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => IRbinaryOperator::Fgreater,
                        (false, true) => IRbinaryOperator::Sgreater,
                        (false, false) => IRbinaryOperator::Ugreater,
                    };
                    (op, RegisterType::Bool, left_t.size())
                },
                ASTbinaryOperator::GreaterEq => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => IRbinaryOperator::FgreaterEq,
                        (false, true) => IRbinaryOperator::SgreaterEq,
                        (false, false) => IRbinaryOperator::UgreaterEq,
                    };
                    (op, RegisterType::Bool, left_t.size())
                },
                ASTbinaryOperator::Eq => {
                    if left_t != right_t {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => IRbinaryOperator::Feq,
                        false => IRbinaryOperator::Eq,
                    };
                    (op, RegisterType::Bool, left_t.size())
                },
                ASTbinaryOperator::NotEq => {
                    if left_t != right_t {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => IRbinaryOperator::FnotEq,
                        false => IRbinaryOperator::FnotEq,
                    };
                    (op, RegisterType::Bool, left_t.size())
                },
                ASTbinaryOperator::Index => todo!(),
            };

            registers.push(op.1);

            instructions.push(Instruction::BinaryOperation { 
                size: op.2, 
                operation: op.0, 
                destination: registers.len() as u16 - 1, 
                left: left_reg, 
                right: right_reg 
            });
            return Some(registers.len() as u16 - 1);
        },
        Expression::UnaryOperator { operator, expr } => {
            let reg = lower_expression(expr, arena, registers, instructions, variables, functions)?;

            let t = registers[reg as usize].clone();

            let op = match operator {
                ASTunaryOperator::Sub => {
                    if !t.is_int() {
                        return None;
                    } 
                    let size = t.size();
                    registers.push(t);
                    instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: 0 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: IRbinaryOperator::Sub, 
                        destination: registers.len() as u16 - 1, 
                        left: registers.len() as u16 - 1 , 
                        right: reg,
                    });
                },
                ASTunaryOperator::Not => {
                    if !t.is_int() {
                        return None;
                    } 
                    let size = t.size();
                    registers.push(t);
                    instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: 1i64.wrapping_shl(size as u32)-1 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: IRbinaryOperator::Xor, 
                        destination: registers.len() as u16 - 1, 
                        left: reg, 
                        right: registers.len() as u16 - 1,
                    });
                },
                ASTunaryOperator::Invert => {
                    if t != RegisterType::Bool {
                        return None;
                    } 
                    let size = t.size();
                    registers.push(t);
                    instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: 1 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: 8, 
                        operation: IRbinaryOperator::Xor, 
                        destination: registers.len() as u16 - 1, 
                        left: reg, 
                        right: registers.len() as u16 - 1,
                    });
                },
                ASTunaryOperator::Dereference => todo!(),
                ASTunaryOperator::Address => todo!(),
            };
            return Some(registers.len() as u16 - 1);
        },
        Expression::Cast { expr, typ } => {
            // TODO: pointer casts

            let left_reg = lower_expression(expr, arena, registers, instructions, variables, functions)?;
            let left_t = registers[left_reg as usize].clone();

            let cast_type = if let Type::Name(n) = typ {
                n
            }
            else { return None; };

            let cast_type = RegisterType::from_str(*cast_type)?;
            let cast_size = cast_type.size();

            if matches!(left_t, RegisterType::Ptr(_))  {
                return None;
            }

            if left_t == RegisterType::Bool && cast_type.is_float() {
                return None;
            }

            if left_t == cast_type {
                return Some(left_reg);
            }

            if left_t.is_float() && cast_type.is_float() {
                registers.push(cast_type);
                instructions.push(Instruction::ExtendTruncate { 
                    typ: if left_t == RegisterType::F32 { ExtTruncType::Fextend } else { ExtTruncType::Ftruncate }, 
                    source: left_reg, 
                    from_size: left_t.size(), 
                    destination: registers.len() as u16 - 1, 
                    to_size: cast_size 
                });
            }
            else if left_t.is_int() && cast_type.is_int() {
                if left_t.size() == cast_type.size() {
                    return Some(left_reg);
                }
                registers.push(cast_type);
                instructions.push(Instruction::ExtendTruncate { 
                    typ: if left_t.size() > cast_size { ExtTruncType::Truncate } else { 
                        if left_t.is_signed() { ExtTruncType::SignExtend } else { ExtTruncType::ZeroExtend } }, 
                    source: left_reg, 
                    from_size: left_t.size(), 
                    destination: registers.len() as u16 - 1, 
                    to_size: cast_size 
                });
            }
            else if left_t.is_float() && cast_type == RegisterType::Bool {
                registers.push( if left_t.size() == 32 { RegisterType::F32 } else { RegisterType::F64 } );
                instructions.push(Instruction::ImmediateFloat { destination: registers.len() as u16 - 1, value: 0.0 });
                registers.push(RegisterType::Bool);

                instructions.push(Instruction::BinaryOperation { 
                    size: left_t.size(), 
                    operation: IRbinaryOperator::FnotEq, 
                    destination: registers.len() as u16 - 1, 
                    left: left_reg, 
                    right: registers.len() as u16 - 2 
                }); 
            }
            else if left_t.is_int() && cast_type.is_float() {
                registers.push(cast_type);
                instructions.push(Instruction::Convert { 
                    typ: if left_t.is_signed() { ConvertType::SignedToFloat } else { ConvertType::UnsignedToFloat }, 
                    source: left_reg, 
                    from_size: left_t.size(), 
                    destination: registers.len() as u16 - 1, 
                    to_size: cast_size, 
                });
            }
            else if left_t.is_float() && cast_type.is_int() {
                let cast_is_signed = cast_type.is_signed();
                registers.push(cast_type);
                instructions.push(Instruction::Convert { 
                    typ: if cast_is_signed { ConvertType::FloatToSigned } else { ConvertType::FloatToUnsigned }, 
                    source: left_reg, 
                    from_size: left_t.size(), 
                    destination: registers.len() as u16 - 1, 
                    to_size: cast_size, 
                });
            }
            else {
                return None;
            }
            return Some(registers.len() as u16 - 1);
        },
        Expression::Call { expr, parameters } => {
            let name = if let Expression::Name(n) = expr {
                *n
            } else { return None; };

            let f = if let Some(f) = functions.get(name) {
                f
            }
            else { return None; };

            if parameters.len() != f.parameters.len() {
                return None;
            }

            let register_paratemers: &mut [u16] = arena.alloc_array(0u16, f.parameters.len());

            for (i, par) in f.parameters.iter().enumerate() {
                let register = lower_expression(&parameters[i], arena, registers, instructions, variables, functions)?;

                let sign_type = registers[register as usize].clone();

                let correct_type = if let Type::Name(n) = f.parameters[i].0 {
                    n
                }
                else { return None; };

                if RegisterType::from_str(correct_type)? != sign_type {
                    return None;
                }

                register_paratemers[i] = register;
            }

            let return_type = if let Some(t) = &f.return_type {
                if let Type::Name(n) = t {
                    registers.push(RegisterType::from_str(n)?);
                    Some(n)
                }
                else { None }
            }
            else { None };

            let register_paratemers: &[u16] = register_paratemers;

            instructions.push(Instruction::Call ( 
                arena.alloc((name, 
                register_paratemers, 
                if return_type.is_some() { Some(registers.len() as u16 -1) } else { None })) 
            ));

            return if return_type.is_some() { Some(registers.len() as u16 -1) } else { Some(u16::MAX) };
        },
        Expression::Field { expr, field } => todo!(),
        Expression::ArrayInit(expressions) => todo!(),
        Expression::StructInit(items) => todo!(),
    }
    return None;
}
fn lower_statement<'a, 'b>(
    statement: &Statement<'a, 'b>,
    arena: &'b Arena,
    registers: &mut Vec<RegisterType<'a, 'b>>, 
    blocks: &mut VecArena<Block<'a, 'b>>,
    variables: &mut Vec<(&'a [u8], u16)>,
    functions: &HashMap<&'a [u8], ASTfunction<'a, 'b>>,
    return_type: Option<RegisterType<'a, 'b>>,
    last_loop_start: Option<i32>,
) -> Option<()> {
    match statement {
        Statement::Assignment { destination, value } => {
            let instructions = &mut blocks.last_mut()?.instructions;
            let value_reg = lower_expression(value, arena, registers, instructions, variables, functions)?;

            if let Some(d) = destination {
                if let Expression::Name(n) = d {
                    let dest_reg = variables.iter().find(|x| &x.0 == n)?;

                    if registers[dest_reg.1 as usize] != registers[value_reg as usize] {
                        return None;
                    }

                    instructions.push(Instruction::Move { source: value_reg, destination: dest_reg.1 });
                }
                else { return None; }
            }
        },
        Statement::Return { value } => {
            let instructions = &mut blocks.last_mut()?.instructions;
            let value_reg = if let Some(v) = value {
                let value_reg = lower_expression(v, arena, registers, instructions, variables, functions)?;
                if return_type.is_none() || return_type.unwrap() != registers[value_reg as usize] {
                    return None;
                }
                Some(value_reg)
            }
            else {
                if return_type.is_some() {
                    return None;
                } 
                None 
            };

            instructions.push(Instruction::Return { value: value_reg });
        },
        Statement::If { condition, body, else_body } => {
            let instructions = &mut blocks.last_mut()?.instructions;
            let c = lower_expression(condition, arena, registers, instructions, variables, functions)?;
            
            if registers[c as usize] != RegisterType::Bool {
                return None;
            }

            registers.push(RegisterType::Bool);
            instructions.push(Instruction::Immediate { destination: registers.len() as u16 -1, value: 1 });
            instructions.push(Instruction::BinaryOperation { 
                size: 8, 
                operation: IRbinaryOperator::Xor, 
                destination: registers.len() as u16 -1, 
                left: registers.len() as u16 -1, 
                right: c, 
            });

            let condition_reg= registers.len() as u16 -1;
            let condition_index = blocks.len()-1;
            blocks.push(Block { instructions: arena.vec_capacity(16) });

            let var_len = variables.len();

            for s in *body {
                lower_statement(s, arena, registers, blocks, variables, functions, return_type.clone(), last_loop_start)?;
            }

            while variables.len() > var_len {
                variables.pop();
            }

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
                    lower_statement(s, arena, registers, blocks, variables, functions, return_type.clone(), last_loop_start)?;
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

            let instructions = &mut blocks.last_mut()?.instructions;
            let c = lower_expression(condition, arena, registers, instructions, variables, functions)?;

            if registers[c as usize] != RegisterType::Bool {
                return None;
            }

            registers.push(RegisterType::Bool);
            instructions.push(Instruction::Immediate { destination: registers.len() as u16 -1, value: 1 });
            instructions.push(Instruction::BinaryOperation { 
                size: 8, 
                operation: IRbinaryOperator::Xor, 
                destination: registers.len() as u16 -1, 
                left: registers.len() as u16 -1, 
                right: c, 
            });

            let condition_reg= registers.len() as u16 -1;

            blocks.push(Block { instructions: arena.vec_capacity(16) });

            let var_len = variables.len();

            for s in *body {
                lower_statement(s, arena, registers, blocks, variables, functions, return_type.clone(), Some(condition_block_index as i32))?;
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
            else { return None; }
        },
        Statement::Declaration { typ, name, value } => {
            let Type::Name(var_type) = typ else { return None; };
            let reg_t = RegisterType::from_str(*var_type)?;

            if let Some(v) = value {
                let value_reg = lower_expression(v, arena, registers, instructions, variables, functions)?;
                if registers[value_reg as usize] != reg_t {
                    return None;
                }
            }

            registers.push(reg_t);

            if variables.iter().find(|x| &x.0 == name).is_some() {
                return None;
            }

            variables.push((name, registers.len() as u16 -1));
        },
        Statement::For { declaration, condition, iteration, body } => {
            let var_len = variables.len();

            if let Some(d) = declaration {
                lower_statement(d, arena, registers, blocks, variables, functions, return_type.clone(), last_loop_start)?;
            }
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let condition_block_index = blocks.len()-1;

            let instructions = &mut blocks.last_mut()?.instructions;
            let condition_reg = if let Some(cond) = condition {
                let c = lower_expression(cond, arena, registers, instructions, variables, functions)?;

                if registers[c as usize] != RegisterType::Bool {
                    return None;
                }

                registers.push(RegisterType::Bool);
                instructions.push(Instruction::Immediate { destination: registers.len() as u16 -1, value: 1 });
                instructions.push(Instruction::BinaryOperation { 
                    size: 8, 
                    operation: IRbinaryOperator::Xor, 
                    destination: registers.len() as u16 -1, 
                    left: registers.len() as u16 -1, 
                    right: c, 
                });

                Some(registers.len() as u16 -1)
            }
            else { None };
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });

            for s in *body {
                lower_statement(s, arena, registers, blocks, variables, functions, return_type.clone(), Some(condition_block_index as i32))?;
            }

            if let Some(ite) = iteration {
                lower_statement(ite, arena, registers, blocks, variables, functions, return_type.clone(), Some(condition_block_index as i32))?;
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
                lower_statement(s, arena, registers, blocks, variables, functions, return_type.clone(), Some(start_block_index as i32))?;
            }

            while variables.len() > var_len {
                variables.pop();
            }

            let instructions = &mut blocks.last_mut()?.instructions;
            let c = lower_expression(condition, arena, registers, instructions, variables, functions)?;

            if registers[c as usize] != RegisterType::Bool {
                return None;
            }

            let next_block_index = blocks.len();

            blocks.get_mut(start_block_index).unwrap().instructions.push(Instruction::Branch { 
                condition: c, 
                label: next_block_index as i32, 
            });

            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
    }
    return Some(());
}