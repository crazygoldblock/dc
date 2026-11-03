#![allow(unused)]

use crate::{arena::Arena, lexer::{TokenError, Tokenizer}};

#[derive(Clone, PartialEq, Debug)]
pub enum Expression<'a, 'b> {
    Integer{
        typ: &'static str,
        value: u64,
    },
    Float{
        typ: &'static str,
        value: f64,
    },
    Bool(bool),
    Name(&'a [u8]),
    Call{
        expr: &'b Expression<'a, 'b>,
        parameters: &'b [Expression<'a, 'b>], 
    },
    BinaryOperator{
        operator: ASTbinaryOperator,
        left: &'b Expression<'a, 'b>,
        right: &'b Expression<'a, 'b>,
    },
    UnaryOperator{
        operator: ASTunaryOperator,
        expr: &'b Expression<'a, 'b>,
    },
    Field{
        expr: &'b Expression<'a, 'b>,
        field: &'a [u8],
    },
    ArrayInit(&'b [Expression<'a, 'b>]),
    StructInit(&'b [(&'a [u8], Expression<'a, 'b>)]),
    Cast{
        expr: &'b Expression<'a, 'b>,
        typ: &'b Type<'a, 'b>,
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum ASTbinaryOperator {
    Add,
    Sub,

    Mul,
    Div,
    Rem,

    And,
    Xor,
    Or,

    Shl,
    Shr,

    Less,
    LessEq,
    Greater,
    GreaterEq,
    Eq,
    NotEq,

    LogicalAnd,
    LogicalOr,

    Index,
}
#[derive(Clone, PartialEq, Debug)]
pub enum ASTunaryOperator {
    Sub,
    Address,
    Not,
    Invert,
    Dereference,
}
#[derive(Clone, PartialEq, Debug)]
pub enum Type<'a, 'b> {
    Name(&'a [u8]),
    Pointer(&'b Type<'a, 'b>),
    Array(&'b Type<'a, 'b>, Expression<'a, 'b>),
}
#[derive(Clone, Debug)]
pub enum Statement<'a, 'b> {
    Assignment{
        destination: Option<(Expression<'a, 'b>)>,
        value: Expression<'a, 'b>,
    },
    Return{
        value: Option<Expression<'a, 'b>>,
    },
    If{
        condition: Expression<'a, 'b>,
        body: &'b [Statement<'a, 'b>], 
        else_body: Option<&'b [Statement<'a, 'b>]>, 
    },
    While{
        condition: Expression<'a, 'b>,
        body: &'b [Statement<'a, 'b>],
    },
    Break,
    Continue,
    Declaration{
        typ: Type<'a, 'b>,
        name: &'a [u8],
        value: Option<Expression<'a, 'b>>,
    },
    For{
        declaration: Option<&'b Statement<'a, 'b>>,
        condition: Option<Expression<'a, 'b>>,
        iteration: Option<&'b Statement<'a, 'b>>,
        body: &'b [Statement<'a, 'b>],
    },
    DoWhile{
        body: &'b [Statement<'a, 'b>],
        condition: Expression<'a, 'b>,
    },
    Switch{
        value: Expression<'a, 'b>,
        arms: &'b [(Expression<'a, 'b>, &'b [Statement<'a, 'b>])],
        default: Option<&'b [Statement<'a, 'b>]>,
    },
}
#[derive(Debug)]
pub struct ASTfunction<'a, 'b> {
    pub public: bool,
    pub return_type: Option<Type<'a, 'b>>,
    pub name: &'a [u8],
    pub parameters: &'b [(Type<'a, 'b>, &'a [u8])],
    pub body: &'b [Statement<'a, 'b>],
}
#[derive(Debug)]
pub struct Constant<'a, 'b> {
    public: bool,
    name: &'a [u8],
    typ: Type<'a, 'b>,
    value: Expression<'a, 'b>,
}
#[derive(PartialEq, Clone, Debug)]
pub struct Struct<'a, 'b> {
    public: bool,
    name: &'a [u8],
    fields: &'b [(Type<'a, 'b>, &'a [u8])],
}
#[derive(PartialEq, Clone, Debug)]
pub struct Union<'a, 'b> {
    public: bool,
    name: &'a [u8],
    fields: &'b [(Type<'a, 'b>, &'a [u8])],
}
#[derive(Debug)]
pub struct Include<'a> {
    path: &'a [u8],
}
#[derive(Debug)]
pub struct GlobalVariable<'a, 'b> {
    public: bool,
    name: &'a [u8],
    typ: Type<'a, 'b>,
    value: Option<Expression<'a, 'b>>,
}
#[derive(Debug)]
pub enum ParseError<'a> {
    EndOfFile,
    UnexpectedEndOfFile{
        line: &'a str,
    },
    ExpectedKeyword{
        expected: &'static str,
        found: &'a str,
        line: &'a str,
    },
    ExpectedSymbol{
        expected: &'static str,
        found: &'a str,
        line: &'a str,
    },
    ExpectedIdentifier{
        found: &'a str,
        line: &'a str,
    },
    ExpectedOperator{
        found: &'a str,
        line: &'a str,
    },
    UnclosedBrackets{
        line: &'a str,
    },
    UnOpenedBrackets{
        line: &'a str,
    },
    ExpectedConstValue{
        line: &'a str,
    },
    ExpectedNumberLiteral{
        line: &'a str,
    },
}
impl<'a> From<TokenError<'a>> for ParseError<'a> {
    fn from(value: TokenError<'a>) -> Self {
        match value {
            TokenError::EndOfFile(l) => ParseError::UnexpectedEndOfFile { line: l },
        }
    }
}
impl<'a> ParseError<'a> {
    pub fn keyword(expected: &'static str, tokens: &mut Tokenizer<'a>) -> ParseError<'a> {
        ParseError::ExpectedKeyword { expected, found: tokens.last(), line: tokens.get_current_line() }
    }
    pub fn symbol(expected: &'static str, tokens: &mut Tokenizer<'a>) -> ParseError<'a> {
        ParseError::ExpectedSymbol { expected, found: tokens.last(), line: tokens.get_current_line() }
    }
    pub fn identifier(tokens: &mut Tokenizer<'a>) -> ParseError<'a> {
        ParseError::ExpectedIdentifier { found: tokens.last(), line: tokens.get_current_line() }
    }
}
#[derive(Debug)]
pub enum Fragment<'a, 'b> {
    ASTfunction(ASTfunction<'a, 'b>),
    Include(Include<'a>),
    Constant(Constant<'a, 'b>),
    Struct(Struct<'a, 'b>),
    Union(Union<'a, 'b>),
    GlobalVariable(GlobalVariable<'a, 'b>),
}
pub fn parse_file<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<Fragment<'a, 'b>, ParseError<'a>> {
    if tokens.peek().is_err() {
        return Err(ParseError::EndOfFile);
    }

    if tokens.peek()? == "include" {
        return Ok(Fragment::Include(parse_include(tokens)?));
    }

    let public = tokens.peek()? == "public";
    if tokens.peek()? == "public" { tokens.next(); }

    if tokens.peek()? == "const" {
        return Ok(Fragment::Constant(parse_constant(tokens, arena, public)?));
    }
    if tokens.peek()? == "struct" {
        return Ok(Fragment::Struct(parse_struct(tokens, arena, public)?));
    }
    if tokens.peek()? == "union" {
        return Ok(Fragment::Union(parse_union(tokens, arena, public)?));
    }
    if tokens.peek()? == "static" {
        return Ok(Fragment::GlobalVariable(parse_global_var(tokens, arena, public)?));
    }
    return Ok(Fragment::ASTfunction(parse_function(tokens, arena, public)?));
}
fn parse_global_var<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Result<GlobalVariable<'a, 'b>, ParseError<'a>> {
    expect_keyword(tokens, "static")?;
    
    let typ = parse_type(tokens, arena)?;
    let name = next_identifier(tokens)?;

    let value = if tokens.next()? == "=" {
        tokens.next();
        Some(parse_expression(tokens, arena)?)
    }
    else { None };

    expect_symbol(tokens, ";")?;
    return Ok(GlobalVariable { public, name: name, typ, value });
}
fn parse_include<'a>(tokens: &mut Tokenizer<'a>) -> Result<Include<'a>, ParseError<'a>> {
    expect_keyword(tokens, "include")?;
    let index = tokens.index();
    while tokens.next()? != ";" {}

    let path = &tokens.buffer()[index..tokens.index()];
    Ok(Include{ path: path })
}
fn parse_function<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Result<ASTfunction<'a, 'b>, ParseError<'a>> {
    let return_type = if tokens.peek()? != "void" {
        Some(parse_type(tokens, &arena)?)
    }
    else { None };

    let name = next_identifier(tokens)?;

    expect_symbol(tokens, "(")?;
    let mut parameters = arena.vec_capacity(16);

    while tokens.peek()? != ")" {
        let t = parse_type(tokens, &arena)?;
        let name = next_identifier(tokens)?;

        parameters.push((t, name));
        if tokens.peek()? == "," { tokens.next(); }
    }
    expect_symbol(tokens, ")")?;
    
    let body = parse_body(tokens, arena)?;

    return Ok(ASTfunction{ 
        public,
        return_type, 
        name: name, 
        parameters: parameters.array(), 
        body
    });
}
fn parse_type<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<Type<'a, 'b>, ParseError<'a>> {
    if tokens.peek()? == "*" {
        tokens.next()?;
        return Ok(Type::Pointer(arena.alloc(parse_type(tokens, arena)?)));
    }
    if tokens.peek()? == "[" {
        tokens.next()?;
        let t = arena.alloc(parse_type(tokens, arena)?);
        
        expect_symbol(tokens, ";")?;
        let len = parse_expression(tokens, arena)?;
        expect_symbol(tokens, "]")?;

        return Ok(Type::Array(t, len));
    }
    return Ok(Type::Name(next_identifier(tokens)?));
}
fn parse_expression<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<Expression<'a, 'b>, ParseError<'a>> {
    let mut bracket_depth = 0;
    let e = parse_binary_expression(tokens, arena, 0, &mut bracket_depth)?;
    if bracket_depth != 0 {
        return Err(ParseError::UnclosedBrackets { line: tokens.get_current_line() });
    }
    Ok(e)
}
fn parse_binary_expression<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, min_precedence: i8, bracket_depth: &mut usize) -> Result<Expression<'a, 'b>, ParseError<'a>> {

    if b"!*&-~".contains(&tokens.peek()?.as_bytes()[0]) { 
        let prefix = tokens.next()?;
        let op = token_to_unary_operator(prefix.as_bytes())
            .ok_or(ParseError::ExpectedOperator { found: tokens.last(), line: tokens.get_current_line() })?;
        let e = parse_binary_expression(tokens, arena, min_precedence, bracket_depth)?;
        return Ok(Expression::UnaryOperator { operator: op, expr: arena.alloc(e) });
    }

    let mut left = if tokens.peek()? == "(" {
        tokens.next();
        *bracket_depth += 1;
        parse_binary_expression(tokens, arena, 0, bracket_depth)?
    }
    else {
        parse_primary(tokens, arena)?
    };

    while let Ok(op_token) = tokens.peek() {
        let precedence = get_operator_precedence(op_token.as_bytes());
        if precedence < min_precedence {
            break;
        }
        let operator = tokens.next()?;

        if operator == "as" {
            left = Expression::Cast { 
                expr: arena.alloc(left), 
                typ: arena.alloc(parse_type(tokens, arena)?),
            };
        }
        else {
            let e = parse_binary_expression(tokens, arena, precedence + 1, bracket_depth)?;

            left = Expression::BinaryOperator {
                operator: token_to_binary_operator(operator.as_bytes()).ok_or(ParseError::ExpectedOperator { found: tokens.last(), line: tokens.get_current_line() })?,
                left: arena.alloc(left),
                right: arena.alloc(e),
            };
        }
    }

    if tokens.peek()? == ")" {
        if *bracket_depth == 0 {
            return Err(ParseError::UnOpenedBrackets { line: tokens.get_current_line() });
        }
        tokens.next();
        *bracket_depth -= 1;
        return Ok(left);
    }
    Ok(left)
}
fn token_to_binary_operator(token: &[u8]) -> Option<ASTbinaryOperator> {
    Some(match token {
        b"+" => ASTbinaryOperator::Add,
        b"-" => ASTbinaryOperator::Sub,
        b"*" => ASTbinaryOperator::Mul,
        b"/" => ASTbinaryOperator::Div,
        b"%" => ASTbinaryOperator::Rem,

        b"|" => ASTbinaryOperator::Or,
        b"^" => ASTbinaryOperator::Xor,
        b"&" => ASTbinaryOperator::And,
        
        b"&&" => ASTbinaryOperator::LogicalAnd,
        b"||" => ASTbinaryOperator::LogicalOr,

        b"==" => ASTbinaryOperator::Eq,
        b"!=" => ASTbinaryOperator::NotEq,
        b"<" => ASTbinaryOperator::Less,
        b"<=" => ASTbinaryOperator::LessEq,
        b">" => ASTbinaryOperator::Greater,
        b">=" => ASTbinaryOperator::GreaterEq,

        b"<<" => ASTbinaryOperator::Shl,
        b">>" => ASTbinaryOperator::Shr,
        _ => return None,
    })
}
fn token_to_unary_operator(token: &[u8]) -> Option<ASTunaryOperator> {
    Some(match token {
        b"-" => ASTunaryOperator::Sub,
        b"*" => ASTunaryOperator::Dereference,
        b"&" => ASTunaryOperator::Address,
        b"!" => ASTunaryOperator::Invert,
        b"~" => ASTunaryOperator::Not,
        _ => return None,
    })
}
fn parse_primary<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<Expression<'a, 'b>, ParseError<'a>> {
    if tokens.peek()? == "[" {
        tokens.next();
        let mut parameters = arena.vec_capacity(16);
        
        while tokens.peek()? != "]" {
            parameters.push(parse_expression(tokens, &arena)?);
            if tokens.peek()? == "," { tokens.next(); }
        }
        expect_symbol(tokens, "]")?;

        return Ok(Expression::ArrayInit(parameters.array()));
    }
    if tokens.peek()? == "{" {
        tokens.next();
        let mut fields = arena.vec_capacity(16);

        while tokens.peek()? != "}" {
            let name = next_identifier(tokens)?;
            expect_symbol(tokens, ":")?;
            let value = parse_expression(tokens, &arena)?;

            fields.push((name, value));
            if tokens.peek()? == "," { tokens.next(); }
        }
        expect_symbol(tokens, "}")?;

        return Ok(Expression::StructInit(fields.array()));
    }
    if tokens.peek()?.as_bytes()[0].is_ascii_digit() {
        let cp = tokens.checkpoint();
        if let Some(i) = parse_integer(tokens) {
            return Ok(Expression::Integer { typ: i.1, value: i.0 });
        }
        tokens.restore(cp);
        if let Some(f) = parse_float(tokens) {
            return Ok(Expression::Float { typ: f.1, value: f.0 });
        }
        return Err(ParseError::ExpectedNumberLiteral { line: tokens.get_current_line() });
    }
    if tokens.peek()? == "true" {
        tokens.next();
        return Ok(Expression::Bool(true));
    }
    if tokens.peek()? == "false" {
        tokens.next();
        return Ok(Expression::Bool(false));
    }

    let mut expr = Expression::Name(next_identifier(tokens)?);

    loop {
        if tokens.peek()? == "(" {
            tokens.next();
            let mut parameters = arena.vec_capacity(16);

            while tokens.peek()? != ")" {
                parameters.push(parse_expression(tokens, arena)?);
                if tokens.peek()? == "," { tokens.next(); }
            }
            tokens.next()?;
 
            expr = Expression::Call {
                expr: arena.alloc(expr),
                parameters: parameters.array(),
            };
        } 
        else if tokens.peek()? == "." {
            tokens.next();

            expr = Expression::Field {
                expr: arena.alloc(expr),
                field: tokens.next()?.as_bytes(),
            };
        }
        else if tokens.peek()? == "[" {
            tokens.next();
            let index = parse_expression(tokens, arena)?;
            expect_symbol(tokens, "]")?;

            expr = Expression::BinaryOperator { 
                operator: ASTbinaryOperator::Index, 
                left: arena.alloc(expr), 
                right: arena.alloc(index) 
            };    
        }
        else {
            break;
        }
    }
    Ok(expr)
}
fn parse_integer(tokens: &mut Tokenizer) -> Option<(u64, &'static str)> {
    fn check(value: i128, signed: bool, max: i128) -> Option<i128> {
        if signed {
            if value > max/2 || value < -(max/2+1) {
                return None;
            }
            return Some(value);
        }
        if value > max {
            return None;
        }
        return Some(value);
    }

    let mut first = tokens.next().ok()?;

    let mut result = 0i128;

    let sign = if first == "-" { -1 } else { 1 }; 
    if first == "-" { first = tokens.next().ok()?; }

    let input = first.as_bytes();

    for i in 0..input.len() {
        if input[i] == b'_' { continue; }

        if !input[i].is_ascii_digit() {
            result *= sign;
            match &input[i..] {
                b"i8" => return Some((check(result, true, u8::MAX as i128)? as u64, "i8")),
                b"i16" => return Some((check(result, true, u16::MAX as i128)? as u64, "i16")),
                b"i32" => return Some((check(result, true, u32::MAX as i128)? as u64, "i32")),
                b"i64" => return Some((check(result, true, u64::MAX as i128)? as u64, "i64")),

                b"u8" => return Some((check(result, false, u8::MAX as i128)? as u64, "u8")),
                b"u16" => return Some((check(result, false, u16::MAX as i128)? as u64, "u16")),
                b"u32" => return Some((check(result, false, u32::MAX as i128)? as u64, "u32")),
                b"u64" => return Some((check(result, false, u64::MAX as i128)? as u64, "u64")),
                _ => return None,
            }
        }
        result += (input[i] - b'0') as i128;
        result *= 10;
    }
    result *= sign;
    return Some((check(result, true, u32::MAX as i128)? as u64, "i32"));
}
fn parse_float(tokens: &mut Tokenizer) -> Option<(f64, &'static str)> {
    let mut first = tokens.next().ok()?;
    
    let sign = if first == "-" { -1.0 } else { 1.0 }; 
    if first == "-" { first = tokens.next().ok()?; }

    let input_left = first.as_bytes();
    let mut result = 0f64;
    for i in 0..input_left.len() {
        if input_left[i] == b'_' { continue; }

        if !input_left[i].is_ascii_digit() {
            return None;
        }
        result += (input_left[i] - b'0') as f64;
        result *= 10.0;
    }

    expect_symbol(tokens, ".").ok()?;

    let input_right = tokens.next().ok()?.as_bytes();
    for i in 0..input_right.len() {
        if input_right[i] == b'_' { continue; }

        if !input_right[i].is_ascii_digit() {
            result *= sign;
            match &input_right[i..] {
                b"f32" => return Some((result, "f32")),
                b"f64" => return Some((result, "f64")),
                _ => return None,
            }
        }
        result += ((input_right[i] - b'0') as f64) * (0.1f64).powi(i as i32 +1);
    }
    result *= sign;
    return Some((result, "f32"));
}
fn get_operator_precedence(op: &[u8]) -> i8 {
    match op {
        b"||" => 1,
        b"&&" => 2,
        b"|" => 3,
        b"^" => 4,
        b"&" => 5,
        b"==" | b"!=" => 6,
        b"<" | b">" | b"<=" | b">=" => 7,
        b"<<" | b">>" => 8,
        b"+" | b"-" => 9,
        b"*" | b"/" | b"%" => 10,
        b"as" => 11,
        _ => -1,
    }
}
fn parse_body<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<&'b [Statement<'a, 'b>], ParseError<'a>> {
    expect_symbol(tokens, "{")?;
    let mut statements = arena.vec_capacity(16);

    while tokens.peek()? != "}" {
        statements.push(parse_statement(tokens, &arena)?);
    }
    expect_symbol(tokens, "}")?;

    Ok(statements.array())
}
fn parse_statement<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<Statement<'a, 'b>, ParseError<'a>> {
    if tokens.peek()? == "return" {
        tokens.next();

        let value = if tokens.peek()? != ";" {
            Some(parse_expression(tokens, arena)?)
        }
        else { None };
        tokens.next();

        return Ok(Statement::Return { value });
    }
    if tokens.peek()? == "break" {
        tokens.next();
        expect_symbol(tokens, ";")?;
        return Ok(Statement::Break);
    }
    if tokens.peek()? == "continue" {
        tokens.next();
        if tokens.next()? != ";" { panic!(); }
        return Ok(Statement::Continue);
    }
    if tokens.peek()? == "while" {
        tokens.next();
        
        let condition = parse_expression(tokens, arena)?;
        let body = parse_body(tokens, arena)?;

        return Ok(Statement::While { condition, body });
    }
    if tokens.peek()? == "if" {
        tokens.next();
        
        let condition = parse_expression(tokens, arena)?;
        let body = parse_body(tokens, arena)?;

        let else_body = if tokens.peek()? == "else" {
            tokens.next();
            Some(parse_body(tokens, arena)?)
        }
        else { None };

        return Ok(Statement::If { condition, body, else_body });
    }
    if tokens.peek()? == "for" {
        tokens.next();
        expect_symbol(tokens, "(")?;

        let declaration = if tokens.peek()? != ";" {
            let d = parse_declaration(tokens, arena)?;
            let r: &Statement<'_, '_> = arena.alloc(Statement::Declaration { typ: d.1, name: d.0, value: d.2 });
            Some(r)
        }
        else { None };

        let condition = if tokens.peek()? != ";" {
            Some(parse_expression(tokens, arena)?)
        }
        else { None };

        expect_symbol(tokens, ";")?;

        let iteration = if tokens.peek()? != ";" {
            let a = parse_assignment(tokens, arena)?;
            let r: &Statement<'_, '_> = arena.alloc(a);
            Some(r)
        }
        else { None };

        expect_symbol(tokens, ";")?;
        expect_symbol(tokens, ")")?;

        let body = parse_body(tokens, arena)?;

        return Ok(Statement::For { declaration, condition, iteration, body });
    }
    if tokens.peek()? == "do" {
        tokens.next();
        let body = parse_body(tokens, arena)?;

        expect_keyword(tokens, "while")?;
        let condition = parse_expression(tokens, arena)?;
        expect_symbol(tokens, ";")?;

        return Ok(Statement::DoWhile { body, condition })
    }
    if tokens.peek()? == "switch" {
        tokens.next()?;
        let value = parse_expression(tokens, arena)?;
        let mut arms = arena.vec_capacity(8);
        let mut default = None;
        while tokens.peek()? != "}" {
            if tokens.peek()? == "case" {
                tokens.next();
                let case = parse_expression(tokens, arena)?;
                expect_symbol(tokens, ":")?;
                let mut body = arena.vec_capacity(4);

                while tokens.peek()? != "case" && tokens.peek()? != "}" {
                    body.push(parse_statement(tokens, arena)?);
                }
                arms.push((case, body.array()));
            }
            else if tokens.peek()? == "default" {
                tokens.next();
                expect_symbol(tokens, ":")?;
                let mut body = arena.vec_capacity(4);
                while tokens.peek()? != "}" {
                    body.push(parse_statement(tokens, arena)?);
                }
                default = Some(body.array());
                break;
            }
            else { break; }
        }
        expect_symbol(tokens, "}")?;
        return Ok(Statement::Switch { value, arms: arms.array(), default  })
    }
    
    let cp = tokens.checkpoint();
    if let Ok(a) = parse_assignment(tokens, arena) {
        return Ok(a);
    }
    tokens.restore(cp);
    
    let d = parse_declaration(tokens, arena)?;
    return Ok(Statement::Declaration { typ: d.1, name: d.0, value: d.2 });
}
fn parse_struct<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Result<Struct<'a, 'b>, ParseError<'a>> {
    expect_keyword(tokens, "struct")?;

    let name = next_identifier(tokens)?;

    let fields = parse_composite_type(tokens, arena)?;

    return Ok(Struct { public, name: name, fields });
}
fn parse_union<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Result<Union<'a, 'b>, ParseError<'a>> {
    expect_keyword(tokens, "union")?;

    let name = next_identifier(tokens)?;

    let fields = parse_composite_type(tokens, arena)?;

    return Ok(Union { public, name: name, fields });
}
fn parse_composite_type<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<&'b [(Type<'a, 'b>, &'a [u8])], ParseError<'a>> {
    let name = next_identifier(tokens)?;

    expect_symbol(tokens, "{");

    let mut fields = arena.vec_capacity(16);

    while tokens.peek()? != "}" {
        if tokens.peek()? == "," {
            tokens.next();
        }

        let t = parse_type(tokens, arena)?;
        let name = next_identifier(tokens)?;
        
        fields.push((t, name));
    }
    expect_symbol(tokens, "}")?;

    return Ok(fields.array());
}
fn parse_constant<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Result<Constant<'a, 'b>, ParseError<'a>> {
    expect_keyword(tokens, "const")?;

    let decl = parse_declaration(tokens, arena)?;
    let value = decl.2.ok_or(ParseError::ExpectedConstValue { line: tokens.get_current_line() })?;

    return Ok(Constant { public, name: decl.0, typ: decl.1, value });
}
fn parse_declaration<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<(&'a [u8], Type<'a, 'b>, Option<Expression<'a, 'b>>), ParseError<'a>> {
    let t = parse_type(tokens, arena)?;
    let name = next_identifier(tokens)?;

    let value = if tokens.peek()? != ";" {
        expect_symbol(tokens, "=")?;
        Some(parse_expression(tokens, arena)?)
    }
    else { None };
    expect_symbol(tokens, ";")?;
    
    Ok((name, t, value))
}
fn parse_assignment<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Result<Statement<'a, 'b>, ParseError<'a>> {
    let first = parse_expression(tokens, arena)?;

    let (destination, value) = if tokens.peek()? != ";" {
        let op = tokens.next()?;
        let mut expr = parse_expression(tokens, arena)?;

        if op != "=" {
            let operator = token_to_binary_operator(&op.as_bytes().split_last().expect("Lexer doesnt return empty tokens").1)
                .ok_or(ParseError::ExpectedOperator { found: tokens.last(), line: tokens.get_current_line() })?;
            expr = Expression::BinaryOperator { operator, left: arena.alloc(first.clone()), right: arena.alloc(expr) };
        }
        (Some(first), expr)
    }
    else { (None, first) };

    expect_symbol(tokens, ";")?;
    Ok(Statement::Assignment { destination, value })
}
fn expect_symbol<'a>(tokens: &mut Tokenizer<'a>, expected: &'static str) -> Result<(), ParseError<'a>> {
    let token = tokens.next()?;
    if token == expected {
        Ok(())
    }
    else {
        Err(ParseError::ExpectedSymbol { expected, found: token.as_str(), line: tokens.get_current_line() })
    }
}
fn expect_keyword<'a>(tokens: &mut Tokenizer<'a>, expected: &'static str) -> Result<(), ParseError<'a>> {
    let token = tokens.next()?;
    if token == expected {
        Ok(())
    }
    else {
        Err(ParseError::ExpectedKeyword { expected, found: token.as_str(), line: tokens.get_current_line() })
    }
}
fn next_identifier<'a>(tokens: &mut Tokenizer<'a>, ) -> Result<&'a [u8], ParseError<'a>> {
    let token = tokens.next()?;

    if !token.as_bytes()[0].is_ascii_alphabetic() {
        return Err(ParseError::ExpectedIdentifier { found: token.as_str(), line: tokens.get_current_line() });
    }
    else {
        Ok(token.as_bytes())
    }
}