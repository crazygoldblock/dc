#![allow(unused)]

use std::{collections::HashMap, ops::{Add, Mul}};

use crate::{arena::Arena, parser::{Expression, Function, Statement, Struct, Type, TypeAlias, Union}};
use crate::parser::ParserBinaryOperator;
use crate::parser::ParserUnaryOperator;

pub struct CompilationUnit<'a, 'b> {
    types: HashMap<&'a str, Type<'a, 'b>>,
    arena: &'b Arena,
}
impl<'a, 'b> CompilationUnit<'a, 'b> {
    pub fn new(arena: &'b Arena) -> CompilationUnit<'a, 'b> {
        CompilationUnit { types: HashMap::new(), arena }
    }
    pub fn compile(&mut self, f: IRfunction<'a, 'b>) -> Vec<Instruction<'a, 'b>> {
        vec![Instruction::Immediate{destination: 3, value: 1}]
    }
}

pub enum Instruction<'a, 'b> {
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
        operation: BinaryOperator,
        destination: u16,
        left: u16,
        right: u16,
    },
    Call{
        start: &'a str,
        parameters: &'b [u16],
        destination: Option<u16>,
    },
    Return{
        size: u8,
        value: u16,
    },
    Branch{
        condition: u16,
        start: &'a str,
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
    }
}
pub enum ConvertType {
    FloatToUnsigned,
    FloatToSigned,

    SignedToFloat,
    UnsignedToFloat,
}
pub enum ExtTruncType {
    ZeroExtend,
    SignExtend,
    Fextend,

    Truncate,
    Ftruncate,
}
pub enum BinaryOperator {
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
pub struct IRfunction<'a, 'b> {
    name: &'a str,

    body: &'b [Instruction<'a, 'b>],
}
#[derive(PartialEq, Clone)]
pub enum VariableType<'a, 'b> {
    Primitive(&'a str),
    Struct(Struct<'a, 'b>),
    Union(Union<'a, 'b>),
    Alias(TypeAlias<'a, 'b>),
}
fn lower_function<'a, 'b>(
    arena: &'b Arena, 
    function: &Function<'a, 'b>, 
    global_vars: &HashMap<&'a str, Type<'a, 'b>>, 
    types: &HashMap<&'a str, VariableType<'a, 'b>>
) -> Option<IRfunction<'a, 'b>> {
    //let mut registers = HashMap::new();

    let a: Statement;


    return None;
}
#[derive(PartialEq, Clone)]
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
            RegisterType::Bool => 1,
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
    fn from_str(input: &'a str) -> Option<RegisterType<'a, 'b>> {
        Some(match input {
            "i8" => RegisterType::I8,
            "i16" => RegisterType::I16,
            "i32" => RegisterType::I32,
            "i64" => RegisterType::I64,
            "u8" => RegisterType::U8,
            "u16" => RegisterType::U16,
            "u32" => RegisterType::U32,
            "u64" => RegisterType::U64,
            "f32" => RegisterType::F32,
            "f64" => RegisterType::F64,
            "u32" => RegisterType::U32,
            "bool" => RegisterType::Bool,
            _ => return None,
        })
    }
}
fn lower_expression<'a, 'b>(
    expression: &Expression<'a, 'b>, 
    arena: &'b Arena, 
    registers: &mut Vec<RegisterType<'a, 'b>>, 
    instructions: &mut Vec<Instruction<'a, 'b>>,
    variables: &mut HashMap<&'a str, u16>,
    functions: &HashMap<&'a str, Function<'a, 'b>>,
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
            return Some(*variables.get(n)?);
        },
        Expression::BinaryOperator { operator, left, right } => {
            let left_reg = lower_expression(left, arena, registers, instructions, variables, functions)?;

            let right_reg = lower_expression(right, arena, registers, instructions, variables, functions)?;

            let left_t = registers[left_reg as usize].clone();
            let right_t = registers[right_reg as usize].clone();

            let op = match operator {
                ParserBinaryOperator::Add => {
                    // TODO: pointer = Add(pointer, int)
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => BinaryOperator::Fadd,
                        false => BinaryOperator::Add,
                    };
                    (op, left_t.clone(), left_t.size())
                },
                ParserBinaryOperator::Sub => {
                    // TODO: pointer = Sub(pointer, int)
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => BinaryOperator::Fsub,
                        false => BinaryOperator::Sub,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ParserBinaryOperator::Mul => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => BinaryOperator::Fmul,
                        false => BinaryOperator::Mul,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ParserBinaryOperator::Div => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => BinaryOperator::Fdiv,
                        (false, true) => BinaryOperator::Sdiv,
                        (false, false) => BinaryOperator::Udiv,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ParserBinaryOperator::Rem => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => BinaryOperator::Frem,
                        (false, true) => BinaryOperator::Srem,
                        (false, false) => BinaryOperator::Urem,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ParserBinaryOperator::And => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) || left_t.is_float() {
                        return None;
                    }
                    let size = left_t.size();
                    (BinaryOperator::And, left_t, size)
                },
                ParserBinaryOperator::Xor => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) || left_t.is_float() {
                        return None;
                    }
                    let size = left_t.size();
                    (BinaryOperator::Xor, left_t, size)
                },
                ParserBinaryOperator::Or => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) || left_t.is_float() {
                        return None;
                    }
                    let size = left_t.size();
                    (BinaryOperator::Or, left_t, size)
                },
                ParserBinaryOperator::Shl => {
                    // TODO: out of bounds shift
                    if (!right_t.is_signed() && !right_t.is_unsigned()) || (!left_t.is_signed() && !left_t.is_unsigned()) {
                        return None;
                    }
                    let size = left_t.size();
                    (BinaryOperator::Shl, left_t, size)
                },
                ParserBinaryOperator::Shr => {
                    // TODO: out of bounds shift
                    if (!right_t.is_signed() && !right_t.is_unsigned()) || (!left_t.is_signed() && !left_t.is_unsigned()) {
                        return None;
                    }

                    let op = match left_t.is_signed() {
                        true => BinaryOperator::ShrSignFill,
                        false => BinaryOperator::ShrZeroFill,
                    };
                    let size = left_t.size();
                    (op, left_t, size)
                },
                ParserBinaryOperator::LogicalAnd => {
                    if left_t != RegisterType::Bool || right_t != RegisterType::Bool {
                        return None;
                    }
                    let size = left_t.size();
                    (BinaryOperator::And, left_t, size)
                },
                ParserBinaryOperator::LogicalOr => {
                    if left_t != RegisterType::Bool || right_t != RegisterType::Bool {
                        return None;
                    }
                    let size = left_t.size();
                    (BinaryOperator::Or, left_t, size)
                },
                ParserBinaryOperator::Less => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => BinaryOperator::Fless,
                        (false, true) => BinaryOperator::Sless,
                        (false, false) => BinaryOperator::Uless,
                    };
                    (op, RegisterType::Bool, RegisterType::Bool.size())
                },
                ParserBinaryOperator::LessEq => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => BinaryOperator::FlessEq,
                        (false, true) => BinaryOperator::SlessEq,
                        (false, false) => BinaryOperator::UlessEq,
                    };
                    (op, RegisterType::Bool, RegisterType::Bool.size())
                },
                ParserBinaryOperator::Greater => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => BinaryOperator::Fgreater,
                        (false, true) => BinaryOperator::Sgreater,
                        (false, false) => BinaryOperator::Ugreater,
                    };
                    (op, RegisterType::Bool, RegisterType::Bool.size())
                },
                ParserBinaryOperator::GreaterEq => {
                    if left_t != right_t || left_t == RegisterType::Bool || matches!(left_t, RegisterType::Ptr(_)) {
                        return None;
                    }

                    let op = match (left_t.is_float(), left_t.is_signed()) {
                        (true, _) => BinaryOperator::FgreaterEq,
                        (false, true) => BinaryOperator::SgreaterEq,
                        (false, false) => BinaryOperator::UgreaterEq,
                    };
                    (op, RegisterType::Bool, RegisterType::Bool.size())
                },
                ParserBinaryOperator::Eq => {
                    if left_t != right_t {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => BinaryOperator::Feq,
                        false => BinaryOperator::Eq,
                    };
                    (op, RegisterType::Bool, RegisterType::Bool.size())
                },
                ParserBinaryOperator::NotEq => {
                    if left_t != right_t {
                        return None;
                    }

                    let op = match left_t.is_float() {
                        true => BinaryOperator::FnotEq,
                        false => BinaryOperator::FnotEq,
                    };
                    (op, RegisterType::Bool, RegisterType::Bool.size())
                },
                ParserBinaryOperator::Index => todo!(),
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
                ParserUnaryOperator::Sub => {
                    if !t.is_signed() || !t.is_unsigned() {
                        return None;
                    } 
                    let size = t.size();
                    registers.push(t);
                    instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: 0 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: BinaryOperator::Sub, 
                        destination: registers.len() as u16 - 1, 
                        left: registers.len() as u16 - 1 , 
                        right: reg,
                    });
                },
                ParserUnaryOperator::Not => {
                    if !t.is_signed() || !t.is_unsigned() {
                        return None;
                    } 
                    let size = t.size();
                    registers.push(t);
                    instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: 1i64.wrapping_shl(size as u32)-1 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: BinaryOperator::Xor, 
                        destination: registers.len() as u16 - 1, 
                        left: reg, 
                        right: registers.len() as u16 - 1,
                    });
                },
                ParserUnaryOperator::Invert => {
                    if t != RegisterType::Bool {
                        return None;
                    } 
                    let size = t.size();
                    registers.push(t);
                    instructions.push(Instruction::Immediate { destination: registers.len() as u16 - 1, value: 1 });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: BinaryOperator::Xor, 
                        destination: registers.len() as u16 - 1, 
                        left: reg, 
                        right: registers.len() as u16 - 1,
                    });
                },
                ParserUnaryOperator::Dereference => todo!(),
                ParserUnaryOperator::Address => todo!(),
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
                    operation: BinaryOperator::FnotEq, 
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

            let register_paratemers = arena.alloc_array(0u16, f.parameters.len());

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

            instructions.push(Instruction::Call { 
                start: name, 
                parameters: register_paratemers, 
                destination: if return_type.is_some() { Some(registers.len() as u16 -1) } else { None }, 
            });

            return if return_type.is_some() { Some(registers.len() as u16 -1) } else { Some(u16::MAX) };
        },
        Expression::Field { expr, field } => todo!(),
        Expression::ArrayInit(expressions) => todo!(),
        Expression::StructInit(items) => todo!(),
    }

    return None;
}
