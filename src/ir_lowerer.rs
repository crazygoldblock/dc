use std::fmt::{self, Debug, Formatter};

use crate::arena::{Arena, VecArena};
use crate::parser::{ASTbinaryOperator, ASTfunction, ASTunaryOperator, Expression, Statement, Struct, Type};

#[derive(Debug)]
pub struct IRfunction<'b> {
    pub paramaters_count: u16,
    pub blocks: &'b [Block<'b>],
    pub stack_size: u32,
}
pub struct Block<'b> {
    pub instructions: VecArena<'b, Instruction<'b>>,
}
impl<'a> Debug for Block<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("\nBlock - instructions: [\n")?;

        for e in self.instructions.array() {
            writeln!(f, "    {:?},", e)?;
        }
        f.write_str("]")
    }
}
#[derive(Debug)]
pub enum Instruction<'b> {
    Immediate{
        destination: u32,
        value: u64,
        size: Size,
    },
    ImmediateFloat{
        long: bool,
        destination: u32,
        value: f64,
    },
    Load{
        size: Size,
        destination: u32,
        address: u32,
    },
    Store{
        size: Size,
        source: u32,
        address: u32,
    },
    BinaryOperation{
        size: Size,
        operation: IRbinaryOperator,
        destination: u32,
        left: u32,
        right: u32,
    },
    Call(&'b ( 
        &'b [u8],
        &'b [u32],
        u32,
        Size),
    ),
    Return{
        value: Option<(u32, Size)>,
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
        from_size: Size,
        destination: u32,
        to_size: Size,
    },
    Convert{
        typ: ConvertType,
        source: u32,
        from_size: Size,
        destination: u32,
        to_size: Size,
    },
    Move{
        size: Size,
        source: u32,
        destination: u32,
    },
    AddrOf{
        source: u32,
        destination: u32,
    },
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Size {
    S8,
    S16,
    S32,
    S64,
}
impl Size {
    pub fn to_offset(&self) -> u32 {
        match self {
            Size::S8 => 1,
            Size::S16 => 2,
            Size::S32 => 4,
            Size::S64 => 8,
        }
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
        !Self::is_float(&self)
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
#[derive(Debug)]
pub enum IRerror<'a> {
    BinaryOperationType{
        operator: ASTbinaryOperator,
        left: RegisterType<'a>,
        right: RegisterType<'a>,
    },
    UnaryOperatorType{
        operator: ASTunaryOperator,
        found: RegisterType<'a>,
    },
    AssignmentType{
        expr: RegisterType<'a>,
        destination: RegisterType<'a>,
    },
    EmptyType,
    ReturnType{
        expected: Option<RegisterType<'a>>,
        found: Option<RegisterType<'a>>,
    },
    ConditionType{
        found: RegisterType<'a>,
    },
    ParameterType{
        expected: RegisterType<'a>,
        found: RegisterType<'a>,
    },
    ArrayElementType{
        expected: RegisterType<'a>,
        found: RegisterType<'a>,
    },
    CastType{
        exp_type: RegisterType<'a>,
        cast_type: RegisterType<'a>,
    },
    CallParametersCount{
        expected: u8,
        found: u8,
    },
    BreakOutsideLoop,
    ContinueOutsideLoop,
    DuplicateVariableName{
        name: &'a[u8],
    },
    DeclarationDestinationType{
        expected: RegisterType<'a>,
        found: RegisterType<'a>,
    },
    LeftsideOpNotDeref,
    LeftsideDerefNotVar,
    LeftsideInvalidExpr,
    VarNameNotFound{
        name: &'a [u8],
    },
    FunctionNotFound{
        name: &'a [u8],
    },
    InvalidType{
        typ: &'a [u8],
    },
    MissignReturn{
        function_name: &'a [u8],
    },
}
pub fn lower_function<'b>(
    arena: &'b Arena,
    function: &ASTfunction,
    functions: &[ASTfunction],
    structs: &[Struct],
) -> Result<IRfunction<'b>, IRerror<'b>> {
    let mut blocks = arena.vec_capacity(8);
    blocks.push(Block { instructions: arena.vec_capacity(16) });

    let mut variables = Vec::new();
    let mut breaks = Vec::new();
    let mut stack_size = 0;
    let parameters = arena.alloc_array(RegisterType::Bool, function.parameters.len());

    for (i, p) in function.parameters.iter().enumerate() {
        let reg_type = type_to_register(&p.0, arena)?;
        parameters[i] = reg_type;
        stack_size += 8;
        variables.push((p.1, stack_size, reg_type));
    }
    stack_size += 8;

    let return_type = if let Some(ref ret_t) = function.return_type {
        Some(type_to_register(ret_t, arena)?)
    }
    else { None };

    for s in function.body {
        lower_statement(s, arena, &mut stack_size, &mut blocks, &mut variables, return_type, None, functions, &mut breaks, structs)?;
    }

    if !matches!(function.body.last().expect("body is not empty"), Statement::Return{value:_}) {
        if function.return_type.is_some() {
            return Err(IRerror::MissignReturn { function_name: arena.alloc_slice(function.name) });
        }
        else {
            lower_statement(&Statement::Return { value: None }, arena, &mut stack_size, &mut blocks, &mut variables, return_type, None, functions, &mut breaks, structs)?;
        }
    }

    if breaks.len() > 0 {
        return Err(IRerror::BreakOutsideLoop);
    }

    return Ok(IRfunction {
        paramaters_count: parameters.len() as u16,
        blocks: blocks.array(),
        stack_size,
    });
}
fn lower_expression<'a, 'b, 'c>(
    expression: &Expression, 
    arena: &'c Arena, 
    stack_size: &mut u32, 
    instructions: &mut VecArena<'c, Instruction<'c>>,
    variables: &mut Vec<(&'a [u8], u32, RegisterType<'c>)>,
    functions: &[ASTfunction],
    structs: &[Struct],
) -> Result<Option<(u32, RegisterType<'c>)>, IRerror<'c>> {

    match expression {
        Expression::Integer{ typ, value } => {
            let typ = type_to_register(&Type::Name(typ.as_bytes()), arena)?;
            stack_push(stack_size, typ.size());
            instructions.push(Instruction::Immediate { destination: *stack_size, value: *value, size: typ.size() });
            return Ok(Some((*stack_size, typ)));
        },
        Expression::Float{ long, value} => {
            let typ_r = if *long { RegisterType::F64 } else { RegisterType::F32 };
            stack_push(stack_size, typ_r.size());
            instructions.push(Instruction::ImmediateFloat { long: *long, destination: *stack_size, value: *value });
            return Ok(Some((*stack_size, typ_r)));
        },
        Expression::Bool(b) => {
            stack_push(stack_size, Size::S8);
            instructions.push(Instruction::Immediate { destination: *stack_size, value: *b as u64, size: Size::S8 });
            return Ok(Some((*stack_size, RegisterType::Bool)));
        },
        Expression::Name(n) => {
            let pos = variables.iter().rev().find(|x| &x.0 == n).ok_or(IRerror::VarNameNotFound { name: arena.alloc_slice(n) })?;
            stack_push(stack_size, pos.2.size());
            instructions.push(Instruction::Move { source: pos.1, destination: *stack_size, size: pos.2.size() });
            return Ok(Some((*stack_size, pos.2)));
        },
        Expression::BinaryOperator { operator, left, right } => {
            let (left_offset, left_type) = lower_expression(left, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            let (mut right_offset, right_type) = lower_expression(right, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            let op = match operator {
                ASTbinaryOperator::Add => {
                    if left_type.is_ptr() && right_type.is_int() && right_type.size() == Size::S64 {
                        stack_push(stack_size, Size::S64);
                        instructions.push(Instruction::Immediate { destination: *stack_size, value: left_type.ptr_element_size().to_offset() as u64, size: Size::S64 });
                           
                        instructions.push(Instruction::BinaryOperation { 
                            size: Size::S64, 
                            operation: IRbinaryOperator::Mul, 
                            destination: *stack_size, 
                            left: *stack_size, 
                            right: right_offset, 
                        });
                        right_offset = *stack_size;
                        (IRbinaryOperator::Add, left_type, Size::S64)
                    }
                    else {
                        if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                            return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Add, left: left_type, right: right_type });
                        }

                        let op = match left_type.is_float() {
                            true => IRbinaryOperator::Fadd,
                            false => IRbinaryOperator::Add,
                        };
                        (op, left_type, left_type.size())
                    }
                },
                ASTbinaryOperator::Sub => {
                    if left_type.is_ptr() && right_type.is_int() && right_type.size() == Size::S64 {
                        stack_push(stack_size, Size::S64);
                        instructions.push(Instruction::Immediate { destination: *stack_size, value: left_type.ptr_element_size().to_offset() as u64, size: Size::S64 });
                           
                        instructions.push(Instruction::BinaryOperation { 
                            size: Size::S64, 
                            operation: IRbinaryOperator::Mul, 
                            destination: *stack_size, 
                            left: *stack_size, 
                            right: right_offset, 
                        });
                        right_offset = *stack_size;
                        (IRbinaryOperator::Sub, left_type, Size::S64)
                    }
                    else {
                        if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                            return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Add, left: left_type, right: right_type });
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
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Mul, left: left_type, right: right_type });
                    }

                    let op = match left_type.is_float() {
                        true => IRbinaryOperator::Fmul,
                        false => IRbinaryOperator::Mul,
                    };
                    let size = left_type.size();
                    (op, left_type, size)
                },
                ASTbinaryOperator::Div => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Div, left: left_type, right: right_type });
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
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Rem, left: left_type, right: right_type });
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
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() || left_type.is_float() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::And, left: left_type, right: right_type });
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::And, left_type, size)
                },
                ASTbinaryOperator::Xor => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() || left_type.is_float() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Xor, left: left_type, right: right_type });
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Xor, left_type, size)
                },
                ASTbinaryOperator::Or => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() || left_type.is_float() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Or, left: left_type, right: right_type });
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Or, left_type, size)
                },
                ASTbinaryOperator::Shl => {
                    // TODO: out of bounds shift
                    if !right_type.is_unsigned() || !left_type.is_int() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Shl, left: left_type, right: right_type });
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Shl, left_type, size)
                },
                ASTbinaryOperator::Shr => {
                    // TODO: out of bounds shift
                    if !right_type.is_unsigned() || !left_type.is_int() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Shl, left: left_type, right: right_type });
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
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::LogicalAnd, left: left_type, right: right_type });
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::And, left_type, size)
                },
                ASTbinaryOperator::LogicalOr => {
                    if left_type != RegisterType::Bool || right_type != RegisterType::Bool {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::LogicalOr, left: left_type, right: right_type });
                    }
                    let size = left_type.size();
                    (IRbinaryOperator::Or, left_type, size)
                },
                ASTbinaryOperator::Less => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Less, left: left_type, right: right_type });
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::Fless,
                        (false, true) => IRbinaryOperator::Sless,
                        (false, false) => IRbinaryOperator::Uless,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::LessEq => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::LessEq, left: left_type, right: right_type });
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::FlessEq,
                        (false, true) => IRbinaryOperator::SlessEq,
                        (false, false) => IRbinaryOperator::UlessEq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::Greater => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Greater, left: left_type, right: right_type });
                    }

                    let op = match (left_type.is_float(), left_type.is_signed()) {
                        (true, _) => IRbinaryOperator::Fgreater,
                        (false, true) => IRbinaryOperator::Sgreater,
                        (false, false) => IRbinaryOperator::Ugreater,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::GreaterEq => {
                    if left_type != right_type || left_type == RegisterType::Bool || left_type.is_ptr() {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::GreaterEq, left: left_type, right: right_type });
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
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Eq, left: left_type, right: right_type });
                    }

                    let op = match left_type.is_float() {
                        true => IRbinaryOperator::Feq,
                        false => IRbinaryOperator::Eq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::NotEq => {
                    if left_type != right_type {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::NotEq, left: left_type, right: right_type });
                    }

                    let op = match left_type.is_float() {
                        true => IRbinaryOperator::FnotEq,
                        false => IRbinaryOperator::NotEq,
                    };
                    (op, RegisterType::Bool, left_type.size())
                },
                ASTbinaryOperator::Index => {
                    if !left_type.is_ptr() || !(right_type.is_int() && right_type.size() == Size::S64) {
                        return Err(IRerror::BinaryOperationType { operator: ASTbinaryOperator::Index, left: left_type, right: right_type });
                    }

                    stack_push(stack_size, Size::S64);
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: left_type.ptr_element_size().to_offset() as u64, size: Size::S64 });
                           
                    instructions.push(Instruction::BinaryOperation { 
                        size: Size::S64, 
                        operation: IRbinaryOperator::Mul, 
                        destination: *stack_size, 
                        left: *stack_size, 
                        right: right_offset, 
                    });
                    right_offset = *stack_size;

                    instructions.push(Instruction::BinaryOperation { 
                        size: Size::S64, 
                        operation: IRbinaryOperator::Add, 
                        destination: *stack_size, 
                        left: left_offset, 
                        right: right_offset, 
                    });

                    instructions.push(Instruction::Load { size: left_type.ptr_element_size(), destination: *stack_size, address: *stack_size });

                    let return_type = if let RegisterType::Ptr(VariableType::Primitive(prim)) = left_type { prim } else { panic!("left type is not a pointer"); };

                    return Ok(Some((*stack_size, *return_type)));
                },
            };

            stack_push(stack_size, op.2);

            instructions.push(Instruction::BinaryOperation { 
                size: op.2, 
                operation: op.0, 
                destination: *stack_size, 
                left: left_offset, 
                right: right_offset, 
            });
            return Ok(Some((*stack_size, op.1)));
        },
        Expression::UnaryOperator { operator, expr } => {
            let (offset, typ) = lower_expression(expr, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            match operator {
                ASTunaryOperator::Sub => {
                    if !typ.is_int() {
                        return Err(IRerror::UnaryOperatorType { operator: ASTunaryOperator::Sub, found: typ });
                    } 
                    let size = typ.size();
                    stack_push(stack_size, size);
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: 0, size });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: IRbinaryOperator::Sub, 
                        destination: *stack_size, 
                        left: *stack_size, 
                        right: offset,
                    });
                    return Ok(Some((*stack_size, typ)));
                },
                ASTunaryOperator::Not => {
                    if !typ.is_int() {
                        return Err(IRerror::UnaryOperatorType { operator: ASTunaryOperator::Not, found: typ });
                    } 
                    let size = typ.size();
                    stack_push(stack_size, size);
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: 1u64.wrapping_shl(size as u32)-1, size });
                    instructions.push(Instruction::BinaryOperation { 
                        size: size, 
                        operation: IRbinaryOperator::Xor, 
                        destination: *stack_size, 
                        left: offset, 
                        right: *stack_size,
                    });
                    return Ok(Some((*stack_size, typ)));
                },
                ASTunaryOperator::Invert => {
                    if typ != RegisterType::Bool {
                        return Err(IRerror::UnaryOperatorType { operator: ASTunaryOperator::Invert, found: typ });
                    } 
                    stack_push(stack_size, typ.size());
                    instructions.push(Instruction::Immediate { destination: *stack_size, value: 1, size: typ.size() });
                    instructions.push(Instruction::BinaryOperation { 
                        size: Size::S8, 
                        operation: IRbinaryOperator::Xor, 
                        destination: *stack_size, 
                        left: offset, 
                        right: *stack_size,
                    });
                    return Ok(Some((*stack_size, typ)));
                },
                ASTunaryOperator::Dereference => {
                    if let RegisterType::Ptr(VariableType::Primitive(prim)) = typ {
                        stack_push(stack_size, prim.size());
                        instructions.push(Instruction::Load { size: typ.ptr_element_size(), destination: *stack_size, address: offset });
                        return Ok(Some((*stack_size, *prim)));
                    }
                    else { 
                        return Err(IRerror::UnaryOperatorType { operator: ASTunaryOperator::Dereference, found: typ });
                    }
                },
                ASTunaryOperator::Address => {
                    let typ = RegisterType::Ptr(VariableType::Primitive(arena.alloc(typ)));
                    stack_push(stack_size, Size::S64);
                    instructions.push(Instruction::AddrOf { source: offset, destination: *stack_size });
                    return Ok(Some((*stack_size, typ)));
                },
            };
        },
        Expression::Cast { expr, typ } => {
            let (offset, left_t) = lower_expression(expr, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            let cast_type = type_to_register(typ, arena)?;
            let cast_size = cast_type.size();

            stack_push(stack_size, cast_size);

            if left_t == cast_type {
                instructions.push(Instruction::Move { source: offset, destination: *stack_size, size: left_t.size() });
            }
            else if left_t.is_ptr() && cast_type.is_ptr() {
                instructions.push(Instruction::Move { source: offset, destination: *stack_size, size: left_t.size() });
            }
            else if left_t.is_float() && cast_type.is_float() {
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
                    instructions.push(Instruction::Move { source: offset, destination: *stack_size, size: left_t.size() });
                }
                else {
                    instructions.push(Instruction::ExtendTruncate { 
                        typ: if left_t.size() as u8 > cast_size as u8 { ExtTruncType::Truncate } else { 
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
                    long: if left_t.size() == Size::S32 { false } else { true }, 
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
                return Err(IRerror::CastType { exp_type: left_t, cast_type });
            }
            return Ok(Some((*stack_size, cast_type)));
        },
        Expression::Call { name, parameters } => {
            let register_paratemers = arena.alloc_array(0u32, parameters.len());
            let function = functions.iter().find(|x| x.name == *name).ok_or(IRerror::FunctionNotFound { name: arena.alloc_slice(name) })?;
            if function.parameters.len() != parameters.len() { 
                return Err(IRerror::CallParametersCount { expected: function.parameters.len()as u8, found: parameters.len()as u8 });
            }

            for (i, par) in parameters.iter().enumerate() {
                let (offset, expression_type) = lower_expression(par, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;
                register_paratemers[i] = offset;
            
                let this_type = type_to_register(&function.parameters[i].0, arena)?;
                if this_type != expression_type {
                    return Err(IRerror::ParameterType { expected: this_type, found: expression_type });
                }  
            }
            
            stack_push(stack_size, Size::S64); // TODO: should this be 64?
            instructions.push(Instruction::Call ( 
                arena.alloc((arena.alloc_slice(name) as &[u8], 
                register_paratemers as &[u32],  
                *stack_size,
                Size::S64,
            ))));
            return match function.return_type.clone() {
                Some(ret_type) => Ok(Some((*stack_size, type_to_register(&ret_type, arena)?))),
                None => Ok(None),
            };
        },
        Expression::Field { expr, field } => {
            let (value_offset, value_type) = 
                lower_expression(expr, arena, stack_size, instructions, variables, functions, structs)?
                .ok_or(IRerror::EmptyType)?;

            if let RegisterType::Ptr(VariableType::Struct(name)) = value_type {
                let st = structs.iter().find(|x| x.name == name).unwrap();
                
                let mut offset = 0u32;
                for f in st.fields {
                    let reg_type = type_to_register(&f.0, arena)?;
                    offset = offset.next_multiple_of(reg_type.size().to_offset());
                    if f.1 == *field {
                        stack_push(stack_size, Size::S64);
                        instructions.push(Instruction::Immediate { destination: *stack_size, value: offset as u64, size: Size::S64 });
                        instructions.push(Instruction::BinaryOperation { size: Size::S64, operation: IRbinaryOperator::Add, destination: *stack_size, left: value_offset, right: *stack_size });
                        return Ok(Some((*stack_size, reg_type)));
                    }
                    offset += reg_type.size().to_offset();
                }
            }
            panic!();
        },
        Expression::ArrayInit(expressions) => {
            let (exp_offset, array_type) = lower_expression(&expressions[0], arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;
            
            stack_push_array(stack_size, array_type.size(), expressions.len() as u32);
            let array_start_offset = *stack_size;

            stack_push(stack_size, Size::S64);
            
            instructions.push(Instruction::AddrOf { source: array_start_offset, destination: *stack_size });
            let array_ptr_offset = *stack_size;

            stack_push(stack_size, Size::S64);
            instructions.push(Instruction::Immediate { destination: *stack_size, value: array_type.size().to_offset() as u64, size: Size::S64 });
            let element_size_offset = *stack_size;

            instructions.push(Instruction::Store { size: array_type.size(), source: exp_offset, address: array_ptr_offset });
            
            for e in expressions[1..].iter() {
                let (exp_offset, exp_type) = lower_expression(e, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

                if exp_type != array_type {
                    return Err(IRerror::ArrayElementType { expected: array_type, found: exp_type });
                }
                
                instructions.push(Instruction::BinaryOperation { size: Size::S64, operation: IRbinaryOperator::Add, destination: array_ptr_offset, left: array_ptr_offset, right: element_size_offset });
                instructions.push(Instruction::Store { size: array_type.size(), source: exp_offset, address: array_ptr_offset });
            }
            instructions.push(Instruction::AddrOf { source: array_start_offset, destination: array_ptr_offset });

            let ret_type = RegisterType::Ptr(VariableType::Primitive(arena.alloc(array_type)));
            return Ok(Some((array_ptr_offset, ret_type)));
        },
        Expression::StructInit { name, fields } => {
            let strct = structs.iter().find(|x| x.name == *name).unwrap();

            let (size, align) = struct_size_align(arena, strct).unwrap();

            *stack_size = stack_size.next_multiple_of(align as u32);
            *stack_size += size;
            let mut struct_stack_start = *stack_size;

            for field_definition in strct.fields {

                let (_, field_value) = fields.iter().find(|x| x.0 == field_definition.1).unwrap();

                let (value_offset, value_type) = 
                    lower_expression(field_value, arena, stack_size, instructions, variables, functions, structs)?
                    .ok_or(IRerror::EmptyType)?;

                let definition_reg = type_to_register(&field_definition.0, arena)?;

                if definition_reg != value_type {
                    panic!();
                }
                struct_stack_start = struct_stack_start.next_multiple_of(definition_reg.size().to_offset());
                let field_offset = struct_stack_start;

                instructions.push(Instruction::Move { size: value_type.size(), source: value_offset, destination: field_offset });
                struct_stack_start += definition_reg.size().to_offset();
            }
            stack_push(stack_size, Size::S64);
            instructions.push(Instruction::AddrOf { source: *stack_size - 8, destination: *stack_size });
            return Ok(Some((*stack_size, RegisterType::Ptr(VariableType::Struct(arena.alloc_slice(strct.name))))));
        },
    }
}
fn struct_size_align<'c>(arena: &'c Arena, s: &Struct) -> Result<(u32, usize), IRerror<'c>> {
    let mut size = 0u32;
    let mut align = 0usize;

    for f in s.fields {
        let reg_type = type_to_register(&f.0, arena)?;
        let curr_size = reg_type.size().to_offset();

        size = size.next_multiple_of(curr_size);
        align = align.max(curr_size as usize);
    }
    return Ok((size.next_multiple_of(align as u32), align));
}
fn lower_statement<'b, 'c>(
    statement: &Statement<'b>,
    arena: &'c Arena,
    stack_size: &mut u32, 
    blocks: &mut VecArena<'c, Block<'c>>,
    variables: &mut Vec<(&'b [u8], u32, RegisterType<'c>)>,
    return_type: Option<RegisterType<'c>>,
    last_loop_start: Option<i32>,
    functions: &[ASTfunction],
    breaks: &mut Vec<(usize, usize)>,
    structs: &[Struct],
) -> Result<(), IRerror<'c>> {
    match statement {
        Statement::Assignment { destination, value } => {
            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let value = lower_expression(value, arena, stack_size, instructions, variables, functions, structs)?;
            
            if let Some(d) = destination {
                let value = value.ok_or(IRerror::EmptyType)?;
                if let Expression::Name(n) = d {
                    let (_, dest_offset, dest_type) = variables.iter().find(|x| &x.0 == n)
                        .ok_or(IRerror::VarNameNotFound { name: arena.alloc_slice(n) })?;

                    if *dest_type != value.1 {
                        return Err(IRerror::AssignmentType { expr: value.1, destination: *dest_type });
                    }

                    instructions.push(Instruction::Move { source: value.0, destination: *dest_offset, size: value.1.size() });
                }
                else if let Expression::UnaryOperator { operator, expr } = d {
                    if *operator != ASTunaryOperator::Dereference {
                        return Err(IRerror::LeftsideOpNotDeref);
                    }
                    if let Expression::Name(n) = expr { 
                        let ptr = variables.iter().find(|x| x.0 == *n)
                            .ok_or(IRerror::VarNameNotFound { name: arena.alloc_slice(n) })?;
                        if !ptr.2.is_ptr() { 
                            return Err(IRerror::UnaryOperatorType { operator: ASTunaryOperator::Dereference, found: ptr.2 });
                        }
                        instructions.push(Instruction::Store { size: value.1.size(), source: value.0, address: ptr.1 });
                    }
                    else { 
                        return Err(IRerror::LeftsideDerefNotVar);
                    }
                }
                else { 
                    return Err(IRerror::LeftsideInvalidExpr);
                }  
            }
        },
        Statement::Return { value } => {
            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let value = match value {
                Some(v) => lower_expression(v, arena, stack_size, instructions, variables, functions, structs)?,
                None => None,
            };
            if value.map(|x| x.1) != return_type {
                return Err(IRerror::ReturnType { expected: return_type, found: value.map(|x| x.1) });
            }
            instructions.push(Instruction::Return { value: value.map(|x| (x.0, x.1.size())) });
        },
        Statement::If { condition, body, else_body } => {
            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let (c_offset, c_type) = lower_expression(condition, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            if c_type != RegisterType::Bool {
                return Err(IRerror::ConditionType { found: c_type });
            }

            stack_push(stack_size, Size::S8);

            instructions.push(Instruction::Immediate { destination: *stack_size, value: 1, size: Size::S8 });
            instructions.push(Instruction::BinaryOperation { 
                size: Size::S8, 
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
                lower_statement(s, arena, stack_size, blocks, variables, return_type, last_loop_start, functions, breaks, structs)?;
            }

            variables.truncate(var_len);

            let next_index = blocks.len();

            blocks.get_mut(condition_index).expect("saved index").instructions.push(Instruction::Branch { 
                condition: condition_reg, 
                label: next_index as i32, 
            });

            if let Some(e) = else_body {
                let jump_index = blocks.len()-1;
                
                blocks.push(Block { instructions: arena.vec_capacity(16) });

                let var_len = variables.len();

                for s in *e {
                    lower_statement(s, arena, stack_size, blocks, variables, return_type, last_loop_start, functions, breaks, structs)?;
                }

                while variables.len() > var_len {
                    variables.pop();
                }

                let next_index = blocks.len();

                blocks.get_mut(jump_index).expect("saved index").instructions.push(Instruction::Jump {  
                    label: next_index as i32, 
                });
            }

            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::While { condition, body } => {
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let condition_block_index = blocks.len()-1;

            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let (c_offset, c_type) = lower_expression(condition, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            if c_type != RegisterType::Bool {
                return Err(IRerror::ConditionType { found: c_type });
            }

            stack_push(stack_size, Size::S8);

            instructions.push(Instruction::Immediate { destination: *stack_size, value: 1, size: Size::S8 });
            instructions.push(Instruction::BinaryOperation { 
                size: Size::S8, 
                operation: IRbinaryOperator::Xor, 
                destination: *stack_size, 
                left: *stack_size, 
                right: c_offset, 
            });

            let condition_reg = *stack_size;

            blocks.push(Block { instructions: arena.vec_capacity(16) });

            let var_len = variables.len();
            let breaks_len = breaks.len();

            for s in *body {
                lower_statement(s, arena, stack_size, blocks, variables, return_type, Some(condition_block_index as i32), functions, breaks, structs)?;
            }

            while variables.len() > var_len {
                variables.pop();
            }

            blocks.last_mut().expect("there is always atleast 1 block").instructions.push(Instruction::Jump { 
                label: condition_block_index as i32, 
            });

            let next_block_index = blocks.len();

            blocks.get_mut(condition_block_index).expect("saved index").instructions.push(Instruction::Branch { 
                condition: condition_reg, 
                label: next_block_index as i32, 
            });

            for (block_index,instruction_index) in &breaks[breaks_len..] {
                *blocks.get_mut(*block_index).expect("saved index").instructions.get_mut(*instruction_index).expect("saved index") 
                    = Instruction::Jump { label: next_block_index as i32 };
            }
            breaks.truncate(breaks_len);
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::Break => {
            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;

            instructions.push(Instruction::Jump { label: i32::MIN });
            breaks.push((blocks.len()-1, blocks.last().expect("not empty").instructions.len()-1));
        },
        Statement::Continue => {
            if let Some(i) = last_loop_start {
                blocks.last_mut().expect("blocks have len of atleast 1").instructions.push(Instruction::Jump { label: i });
            }
            else { 
                return Err(IRerror::ContinueOutsideLoop);
            }
        },
        Statement::Declaration { typ, name, value } => {
            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let reg_t = type_to_register(typ, arena)?;

            if variables.iter().rev().find(|x| &x.0 == name).is_some() {
                return Err(IRerror::DuplicateVariableName { name: arena.alloc_slice(name) });
            }

            if let Some(v) = value {
                let (value_offset, value_type) = lower_expression(v, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;
                
                if value_type != reg_t {
                    return Err(IRerror::DeclarationDestinationType { expected: reg_t, found: value_type });
                }
                variables.push((name, value_offset, reg_t));
            }
            else {
                stack_push(stack_size, reg_t.size());
                variables.push((name, *stack_size, reg_t));
            }
        },
        Statement::For { declaration, condition, iteration, body } => {
            let var_len = variables.len();
            let breaks_len = breaks.len();

            if let Some(d) = declaration {
                lower_statement(d, arena, stack_size, blocks, variables, return_type, last_loop_start, functions, breaks, structs)?;
            }
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let condition_block_index = blocks.len()-1;

            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let condition_reg = if let Some(cond) = condition {
                let (c_offset, c_type) = lower_expression(cond, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

                if c_type != RegisterType::Bool {
                    return Err(IRerror::ConditionType { found: c_type });
                }

                stack_push(stack_size, Size::S8);

                instructions.push(Instruction::Immediate { destination: *stack_size, value: 1, size: Size::S8 });
                instructions.push(Instruction::BinaryOperation { 
                    size: Size::S8, 
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
                lower_statement(s, arena, stack_size, blocks, variables, return_type, Some(condition_block_index as i32), functions, breaks, structs)?;
            }

            if let Some(ite) = iteration {
                lower_statement(ite, arena, stack_size, blocks, variables, return_type, Some(condition_block_index as i32), functions, breaks, structs)?;
            }

            while variables.len() > var_len {
                variables.pop();
            }

            blocks.last_mut().expect("blocks have len of atleast 1").instructions.push(Instruction::Jump { 
                label: condition_block_index as i32, 
            });

            let next_block_index = blocks.len();

            if let Some(cond_reg) = condition_reg {
                blocks.get_mut(condition_block_index).expect("saved index").instructions.push(Instruction::Branch { 
                    condition: cond_reg, 
                    label: next_block_index as i32, 
                });
            }

            for (block_index, instruction_index) in &breaks[breaks_len..] {
                *blocks.get_mut(*block_index).expect("saved index").instructions.get_mut(*instruction_index).expect("saved index") 
                    = Instruction::Jump { label: next_block_index as i32 };
            }
            breaks.truncate(breaks_len);
            
            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::DoWhile { body, condition } => {
            blocks.push(Block { instructions: arena.vec_capacity(16) });
            let start_block_index = blocks.len()-1;

            let var_len = variables.len();
            let breaks_len = breaks.len();

            for s in *body {
                lower_statement(s, arena, stack_size, blocks, variables, return_type, Some(start_block_index as i32), functions, breaks, structs)?;
            }

            while variables.len() > var_len {
                variables.pop();
            }

            let instructions = &mut blocks.last_mut().expect("blocks have len of atleast 1").instructions;
            let (c_offset, c_type) = lower_expression(condition, arena, stack_size, instructions, variables, functions, structs)?.ok_or(IRerror::EmptyType)?;

            if c_type != RegisterType::Bool {
                return Err(IRerror::ConditionType { found: c_type });
            }

            let next_block_index = blocks.len();

            blocks.get_mut(start_block_index).expect("saved index").instructions.push(Instruction::Branch { 
                condition: c_offset, 
                label: next_block_index as i32, 
            });

            for (block_index,instruction_index) in &breaks[breaks_len..] {
                *blocks.get_mut(*block_index).expect("saved index").instructions.get_mut(*instruction_index).expect("saved index") 
                    = Instruction::Jump { label: next_block_index as i32 };
            }
            breaks.truncate(breaks_len);

            blocks.push(Block { instructions: arena.vec_capacity(16) });
        },
        Statement::Switch { value: _, arms: _, default: _ } => todo!(),
    }
    return Ok(());
}
fn type_to_register<'b>(typ: &Type, arena: &'b Arena) -> Result<RegisterType<'b>, IRerror<'b>> {
    match typ {
        Type::Name(name) => {
            RegisterType::from_str(arena.alloc_slice(name)).ok_or(IRerror::InvalidType { typ: arena.alloc_slice(name) })
        },
        Type::Pointer(ptr) => Ok(RegisterType::Ptr(VariableType::Primitive(arena.alloc(type_to_register(ptr, arena)?)))),
    }
}
#[derive(PartialEq, Clone, Debug, Copy)]
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
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum VariableType<'b> {
    Primitive(&'b RegisterType<'b>),
    Struct(&'b [u8]),
    //Union, //(Union<'a, 'b>),
}
impl<'b> RegisterType<'b> {
    fn size(&self) -> Size {
        match self {
            RegisterType::I8 => Size::S8,
            RegisterType::I16 => Size::S16,
            RegisterType::I32 => Size::S32,
            RegisterType::I64 => Size::S64,
            RegisterType::U8 => Size::S8,
            RegisterType::U16 => Size::S16,
            RegisterType::U32 => Size::S32,
            RegisterType::U64 => Size::S64,
            RegisterType::F32 => Size::S32,
            RegisterType::F64 => Size::S64,
            RegisterType::Bool => Size::S8,
            RegisterType::Ptr(_) => Size::S64,
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
            b"bool" => RegisterType::Bool,
            _ => return None,
        })
    }
    fn is_ptr(&self) -> bool {
        matches!(self, RegisterType::Ptr(_))
    }
    fn ptr_element_size(&self) -> Size {
        if let RegisterType::Ptr(ptr) = self {
            if let VariableType::Primitive(prim) = ptr {
                return prim.size();
            }
        }
        panic!("type is not a pointer");
    }
    pub fn to_string(&self) -> String {
        match self {
            RegisterType::I8 => return "i8".to_string(),
            RegisterType::I16 => return  "i16".to_string(),
            RegisterType::I32 => return  "i32".to_string(),
            RegisterType::I64 => return  "i64".to_string(),
            RegisterType::U8 => return  "u8".to_string(),
            RegisterType::U16 => return  "u16".to_string(),
            RegisterType::U32 => return  "u32".to_string(),
            RegisterType::U64 => return  "u64".to_string(),
            RegisterType::F32 => return  "f32".to_string(),
            RegisterType::F64 => return  "f64".to_string(),
            RegisterType::Bool => return  "bool".to_string(),
            RegisterType::Ptr(variable_type) => {
                match variable_type {
                    VariableType::Primitive(register_type) => "*".to_string() + &register_type.to_string(),
                    VariableType::Struct(name) => format!("Struct: {}", String::from_utf8_lossy(name)),
                }
            },
        }
    }
}
fn stack_push(stack_size: &mut u32, size: Size) {
    *stack_size += size.to_offset();
    *stack_size = stack_size.next_multiple_of(size.to_offset());
}
fn stack_push_array(stack_size: &mut u32, size: Size, len: u32) {
    *stack_size += size.to_offset() * len;
    *stack_size = stack_size.next_multiple_of(size.to_offset());
}