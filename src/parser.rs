use crate::{arena::Arena, lexer::{TokenError, Lexer}};

#[derive(Clone, PartialEq, Debug)]
pub enum Expression<'b> {
    Integer{
        typ: &'static str,
        value: u64,
    },
    Float{
        long: bool,
        value: f64,
    },
    Bool(bool),
    Name(&'b [u8]),
    Call{
        name: &'b [u8],
        parameters: &'b [Expression<'b>], 
    },
    BinaryOperator{
        operator: ASTbinaryOperator,
        left: &'b Expression<'b>,
        right: &'b Expression<'b>,
    },
    UnaryOperator{
        operator: ASTunaryOperator,
        expr: &'b Expression<'b>,
    },
    Field{
        expr: &'b Expression<'b>,
        field: &'b [u8],
    },
    ArrayInit(&'b [Expression<'b>]),
    StructInit{
        name: &'b [u8], 
        fields: &'b [(&'b [u8], Expression<'b>)]
    },
    Cast{
        expr: &'b Expression<'b>,
        typ: &'b Type<'b>,
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
pub enum Type<'b> {
    Name(&'b [u8]),
    Pointer(&'b Type<'b>),
}
#[derive(Clone, Debug)]
pub enum Statement<'b> {
    Assignment{
        destination: Option<Expression<'b>>,
        value: Expression<'b>,
    },
    Return{
        value: Option<Expression<'b>>,
    },
    If{
        condition: Expression<'b>,
        body: &'b [Statement<'b>], 
        else_body: Option<&'b [Statement<'b>]>, 
    },
    While{
        condition: Expression<'b>,
        body: &'b [Statement<'b>],
    },
    Break,
    Continue,
    Declaration{
        typ: Type<'b>,
        name: &'b [u8],
        value: Option<Expression<'b>>,
    },
    For{
        declaration: Option<&'b Statement<'b>>,
        condition: Option<Expression<'b>>,
        iteration: Option<&'b Statement<'b>>,
        body: &'b [Statement<'b>],
    },
    DoWhile{
        body: &'b [Statement<'b>],
        condition: Expression<'b>,
    },
    #[allow(unused)]
    Switch{
        value: Expression<'b>,
        arms: &'b [(Expression<'b>, &'b [Statement<'b>])],
        default: Option<&'b [Statement<'b>]>,
    },
}
#[derive(Debug)]
pub struct ASTfunction<'b> {
    #[allow(unused)]
    pub public: bool,
    #[allow(unused)]
    pub path: &'b [u8],
    pub return_type: Option<Type<'b>>,
    pub name: &'b [u8],
    pub parameters: &'b [(Type<'b>, &'b [u8])],
    pub body: &'b [Statement<'b>],
}
#[derive(Debug)]
#[allow(unused)]
pub struct Constant<'b> {
    public: bool,
    name: &'b [u8],
    typ: Type<'b>,
    value: Expression<'b>,
}
#[derive(PartialEq, Clone, Debug)]
pub struct Struct<'b> {
    pub public: bool,
    pub name: &'b [u8],
    pub fields: &'b [(Type<'b>, &'b [u8])],
}
#[derive(PartialEq, Clone, Debug)]
pub struct Union<'b> {
    public: bool,
    name: &'b [u8],
    fields: &'b [(Type<'b>, &'b [u8])],
}
#[derive(Debug)]
#[allow(unused)]
pub struct Include<'a> {
    path: &'a [u8],
}
#[derive(Debug)]
#[allow(unused)]
pub struct GlobalVariable<'b> {
    public: bool,
    name: &'b [u8],
    typ: Type<'b>,
    value: Option<Expression<'b>>,
}
#[derive(Debug)]
#[allow(unused)]
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
impl<'a> From<TokenError> for ParseError<'a> {
    fn from(_value: TokenError) -> Self {
        ParseError::UnexpectedEndOfFile { line: "" }
    }
}
#[derive(Debug)]
#[allow(unused)]
pub enum Fragment<'b> {
    ASTfunction(ASTfunction<'b>),
    Include(Include<'b>),
    Constant(Constant<'b>),
    Struct(Struct<'b>),
    Union(Union<'b>),
    GlobalVariable(GlobalVariable<'b>),
}
pub fn parse_file<'b>(tokens: &mut Lexer, arena: &'b Arena, path: &str) -> Result<Fragment<'b>, ParseError<'b>> {
    if tokens.peek().is_err() {
        return Err(ParseError::EndOfFile);
    }

    if tokens.peek()? == "include" {
        return Ok(Fragment::Include(parse_include(tokens, arena)?));
    }

    let public = tokens.peek()? == "public";
    if tokens.peek()? == "public" { tokens.next().unwrap(); }

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
    return Ok(Fragment::ASTfunction(parse_function(tokens, arena, public, path.as_bytes())?));
}
fn parse_global_var<'b>(tokens: &mut Lexer, arena: &'b Arena, public: bool) -> Result<GlobalVariable<'b>, ParseError<'b>> {
    expect_keyword(tokens, arena, "static")?;
    
    let typ = parse_type(tokens, arena)?;
    let name = next_identifier(tokens, arena)?;

    let value = if tokens.next()? == "=" {
        tokens.next().unwrap();
        Some(parse_expression(tokens, arena)?)
    }
    else { None };

    expect_symbol(tokens, arena, ";")?;
    return Ok(GlobalVariable { public, name: name, typ, value });
}
fn parse_include<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<Include<'b>, ParseError<'b>> {
    expect_keyword(tokens, arena, "include")?;
    let index = tokens.index();
    while tokens.next()? != ";" {}

    let path = &tokens.buffer()[index..tokens.index()];
    Ok(Include{ path: arena.alloc_slice(path) })
}
fn parse_function<'a, 'b>(tokens: &mut Lexer<'a>, arena: &'b Arena, public: bool, path: &[u8]) -> Result<ASTfunction<'b>, ParseError<'b>> {
    let return_type = if tokens.peek()? != "void" {
        Some(parse_type(tokens, &arena)?)
    }
    else {
        tokens.next()?; 
        None 
    };

    let name = next_identifier(tokens, arena)?;

    expect_symbol(tokens, arena, "(")?;
    let mut parameters = arena.vec_capacity(16);

    while tokens.peek()? != ")" {
        let t = parse_type(tokens, &arena)?;
        let name = next_identifier(tokens, arena)?;

        parameters.push((t, name));
        if tokens.peek()? == "," { tokens.next().unwrap(); }
    }
    expect_symbol(tokens, arena, ")")?;
    
    let body = parse_body(tokens, arena)?;

    return Ok(ASTfunction{ 
        public,
        path: arena.alloc_slice(path),
        return_type, 
        name: name, 
        parameters: parameters.array(), 
        body
    });
}
fn parse_type<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<Type<'b>, ParseError<'b>> {
    if tokens.peek()? == "*" {
        tokens.next()?;
        return Ok(Type::Pointer(arena.alloc(parse_type(tokens, arena)?)));
    }
    return Ok(Type::Name(next_identifier(tokens, arena)?));
}
fn parse_expression<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<Expression<'b>, ParseError<'b>> {
    let mut bracket_depth = 0;
    let e = parse_binary_expression(tokens, arena, 0, &mut bracket_depth)?;
    if bracket_depth != 0 {
        return Err(ParseError::UnclosedBrackets { line: arena.alloc_str(tokens.get_current_line()) });
    }
    Ok(e)
}
fn parse_binary_expression<'b>(tokens: &mut Lexer, arena: &'b Arena, min_precedence: i8, bracket_depth: &mut usize) -> Result<Expression<'b>, ParseError<'b>> {

    let mut left = if tokens.peek()? == "(" {
        tokens.next().unwrap();
        *bracket_depth += 1;
        parse_binary_expression(tokens, arena, 0, bracket_depth)?
    }
    else {
        if b"!*&-~".contains(&tokens.peek()?.as_bytes()[0]) { 
            let prefix = tokens.next()?;
            let op = token_to_unary_operator(prefix.as_bytes())
                .ok_or(ParseError::ExpectedOperator { found: arena.alloc_str(tokens.last()), line: arena.alloc_str(tokens.get_current_line()) })?;
            let e = parse_binary_expression(tokens, arena, i8::MAX, bracket_depth)?;
            Expression::UnaryOperator { operator: op, expr: arena.alloc(e) }
        }
        else {
            parse_primary(tokens, arena)?
        }
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
                operator: token_to_binary_operator(operator.as_bytes()).ok_or(ParseError::ExpectedOperator { found: arena.alloc_str(tokens.last()), line: arena.alloc_str(tokens.get_current_line()) })?,
                left: arena.alloc(left),
                right: arena.alloc(e),
            };
        }
    }

    if tokens.peek()? == ")" && *bracket_depth > 0 {
        tokens.next().unwrap();
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
fn parse_primary<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<Expression<'b>, ParseError<'b>> {
    if tokens.peek()? == "[" {
        tokens.next().unwrap();
        let mut parameters = arena.vec_capacity(16);

        parameters.push(parse_expression(tokens, &arena)?);
        if tokens.peek()? == "," { tokens.next().unwrap(); }
        
        while tokens.peek()? != "]" {
            parameters.push(parse_expression(tokens, &arena)?);
            if tokens.peek()? == "," { tokens.next().unwrap(); }
        }
        expect_symbol(tokens, arena, "]")?;

        return Ok(Expression::ArrayInit(parameters.array()));
    }
    if tokens.peek()?.as_bytes()[0].is_ascii_digit() {
        let cp = tokens.checkpoint();
        if let Some(i) = parse_integer(tokens) {
            return Ok(Expression::Integer { typ: i.1, value: i.0 });
        }
        tokens.restore(cp);
        if let Some(f) = parse_float(tokens, arena) {
            return Ok(Expression::Float { long: f.1, value: f.0 });
        }
        return Err(ParseError::ExpectedNumberLiteral { line: arena.alloc_str(tokens.get_current_line()) });
    }
    if tokens.peek()? == "true" {
        tokens.next().unwrap();
        return Ok(Expression::Bool(true));
    }
    if tokens.peek()? == "false" {
        tokens.next().unwrap();
        return Ok(Expression::Bool(false));
    }

    let identifier = next_identifier(tokens, arena)?;

    if tokens.peek()? == "{" {
        tokens.next().unwrap();
        let mut fields = arena.vec_capacity(16);

        while tokens.peek()? != "}" {
            let name = next_identifier(tokens, arena)?;
            expect_symbol(tokens, arena, ":")?;
            let value = parse_expression(tokens, &arena)?;

            fields.push((name, value));
            if tokens.peek()? == "," { tokens.next().unwrap(); }
        }
        expect_symbol(tokens, arena, "}")?;

        return Ok(Expression::StructInit{ name: identifier, fields: fields.array()});
    }

    let mut expr = Expression::Name(identifier);

    if tokens.peek()? == "(" {
        tokens.next().unwrap();
        let mut parameters = arena.vec_capacity(16);

        while tokens.peek()? != ")" {
            parameters.push(parse_expression(tokens, arena)?);
            if tokens.peek()? == "," { tokens.next().unwrap(); }
        }
        tokens.next()?;
 
        expr = Expression::Call {
            name: identifier,
            parameters: parameters.array(),
        };
    } 

    loop {
        if tokens.peek()? == "." {
            tokens.next().unwrap();

            expr = Expression::Field {
                expr: arena.alloc(expr),
                field: arena.alloc_slice(tokens.next()?.as_bytes()),
            };
        }
        else if tokens.peek()? == "[" {
            tokens.next().unwrap();
            let index = parse_expression(tokens, arena)?;
            expect_symbol(tokens, arena, "]")?;

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
fn parse_integer(tokens: &mut Lexer) -> Option<(u64, &'static str)> {
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
        result *= 10;
        result += (input[i] - b'0') as i128;
    }
    result *= sign;
    return Some((check(result, true, u32::MAX as i128)? as u64, "i32"));
}
fn parse_float(tokens: &mut Lexer, arena: &Arena) -> Option<(f64, bool)> {
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

    expect_symbol(tokens, arena,  ".").ok()?;

    let input_right = tokens.next().ok()?.as_bytes();
    for i in 0..input_right.len() {
        if input_right[i] == b'_' { continue; }

        if !input_right[i].is_ascii_digit() {
            result *= sign;
            match &input_right[i..] {
                b"f32" => return Some((result, false)),
                b"f64" => return Some((result, true)),
                _ => return None,
            }
        }
        result += ((input_right[i] - b'0') as f64) * (0.1f64).powi(i as i32 +1);
    }
    result *= sign;
    return Some((result, false));
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
fn parse_body<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<&'b [Statement<'b>], ParseError<'b>> {
    expect_symbol(tokens, arena, "{")?;
    let mut statements = arena.vec_capacity(16);

    while tokens.peek()? != "}" {
        statements.push(parse_statement(tokens, &arena)?);
    }
    expect_symbol(tokens, arena, "}")?;

    Ok(statements.array())
}
fn parse_statement<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<Statement<'b>, ParseError<'b>> {
    if tokens.peek()? == "return" {
        tokens.next().unwrap();

        let value = if tokens.peek()? != ";" {
            Some(parse_expression(tokens, arena)?)
        }
        else { None };
        tokens.next().unwrap();

        return Ok(Statement::Return { value });
    }
    if tokens.peek()? == "break" {
        tokens.next().unwrap();
        expect_symbol(tokens, arena, ";")?;
        return Ok(Statement::Break);
    }
    if tokens.peek()? == "continue" {
        tokens.next().unwrap();
        if tokens.next()? != ";" { panic!(); }
        return Ok(Statement::Continue);
    }
    if tokens.peek()? == "while" {
        tokens.next().unwrap();
        
        let condition = parse_expression(tokens, arena)?;
        let body = parse_body(tokens, arena)?;

        return Ok(Statement::While { condition, body });
    }
    if tokens.peek()? == "if" {
        tokens.next().unwrap();
        
        let condition = parse_expression(tokens, arena)?;
        let body = parse_body(tokens, arena)?;

        let else_body = if tokens.peek()? == "else" {
            tokens.next().unwrap();
            Some(parse_body(tokens, arena)?)
        }
        else { None };

        return Ok(Statement::If { condition, body, else_body });
    }
    if tokens.peek()? == "for" {
        tokens.next().unwrap();
        expect_symbol(tokens, arena, "(")?;

        let declaration = if tokens.peek()? != ";" {
            let d = parse_declaration(tokens, arena)?;
            let r: &Statement<'_> = arena.alloc(Statement::Declaration { typ: d.1, name: d.0, value: d.2 });
            Some(r)
        }
        else {
            tokens.next()?; 
            None 
        };

        let condition = if tokens.peek()? != ";" {
            Some(parse_expression(tokens, arena)?)
        }
        else { None };

        expect_symbol(tokens, arena, ";")?;

        let iteration = if tokens.peek()? != ";" {
            let a = parse_assignment(tokens, arena)?;
            let r: &Statement<'_> = arena.alloc(a);
            Some(r)
        }
        else { 
            tokens.next()?; 
            None 
        };

        expect_symbol(tokens, arena, ")")?;

        let body = parse_body(tokens, arena)?;

        return Ok(Statement::For { declaration, condition, iteration, body });
    }
    if tokens.peek()? == "do" {
        tokens.next().unwrap();
        let body = parse_body(tokens, arena)?;

        expect_keyword(tokens, arena, "while")?;
        let condition = parse_expression(tokens, arena)?;
        expect_symbol(tokens, arena, ";")?;

        return Ok(Statement::DoWhile { body, condition })
    }
    if tokens.peek()? == "switch" {
        tokens.next()?;
        let value = parse_expression(tokens, arena)?;
        let mut arms = arena.vec_capacity(8);
        let mut default = None;
        while tokens.peek()? != "}" {
            if tokens.peek()? == "case" {
                tokens.next().unwrap();
                let case = parse_expression(tokens, arena)?;
                expect_symbol(tokens, arena, ":")?;
                let mut body = arena.vec_capacity(4);

                while tokens.peek()? != "case" && tokens.peek()? != "}" {
                    body.push(parse_statement(tokens, arena)?);
                }
                arms.push((case, body.array()));
            }
            else if tokens.peek()? == "default" {
                tokens.next().unwrap();
                expect_symbol(tokens, arena, ":")?;
                let mut body = arena.vec_capacity(4);
                while tokens.peek()? != "}" {
                    body.push(parse_statement(tokens, arena)?);
                }
                default = Some(body.array());
                break;
            }
            else { break; }
        }
        expect_symbol(tokens, arena, "}")?;
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
fn parse_struct<'b>(tokens: &mut Lexer, arena: &'b Arena, public: bool) -> Result<Struct<'b>, ParseError<'b>> {
    expect_keyword(tokens, arena, "struct")?;

    let name = next_identifier(tokens, arena)?;

    let fields = parse_composite_type(tokens, arena)?;

    return Ok(Struct { public, name: name, fields });
}
fn parse_union<'b>(tokens: &mut Lexer, arena: &'b Arena, public: bool) -> Result<Union<'b>, ParseError<'b>> {
    expect_keyword(tokens, arena, "union")?;

    let name = next_identifier(tokens, arena)?;

    let fields = parse_composite_type(tokens, arena)?;

    return Ok(Union { public, name: name, fields });
}
fn parse_composite_type<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<&'b [(Type<'b>, &'b [u8])], ParseError<'b>> {
    expect_symbol(tokens, arena, "{")?;

    let mut fields = arena.vec_capacity(16);

    while tokens.peek()? != "}" {
        if tokens.peek()? == "," {
            tokens.next().unwrap();
        }

        let t = parse_type(tokens, arena)?;
        let name = next_identifier(tokens, arena)?;
        
        fields.push((t, name));
    }
    expect_symbol(tokens, arena, "}")?;

    return Ok(fields.array());
}
fn parse_constant<'b>(tokens: &mut Lexer, arena: &'b Arena, public: bool) -> Result<Constant<'b>, ParseError<'b>> {
    expect_keyword(tokens, arena, "const")?;

    let decl = parse_declaration(tokens, arena)?;
    let value = decl.2.ok_or(ParseError::ExpectedConstValue { line: arena.alloc_str(tokens.get_current_line()) })?;

    return Ok(Constant { public, name: decl.0, typ: decl.1, value });
}
fn parse_declaration<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<(&'b [u8], Type<'b>, Option<Expression<'b>>), ParseError<'b>> {
    let t = parse_type(tokens, arena)?;
    let name = next_identifier(tokens, arena)?;

    let value = if tokens.peek()? != ";" {
        expect_symbol(tokens, arena, "=")?;
        Some(parse_expression(tokens, arena)?)
    }
    else { None };
    expect_symbol(tokens, arena, ";")?;
    
    Ok((name, t, value))
}
fn parse_assignment<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<Statement<'b>, ParseError<'b>> {
    let first = parse_expression(tokens, arena)?;

    let (destination, value) = if tokens.peek()? != ";" {
        let op = tokens.next()?;
        let mut expr = parse_expression(tokens, arena)?;

        if op != "=" {
            let operator = token_to_binary_operator(&op.as_bytes().split_last().expect("Lexer doesnt return empty tokens").1)
                .ok_or(ParseError::ExpectedOperator { found: arena.alloc_str(tokens.last()), line: arena.alloc_str(tokens.get_current_line()) })?;
            expr = Expression::BinaryOperator { operator, left: arena.alloc(first.clone()), right: arena.alloc(expr) };
        }
        (Some(first), expr)
    }
    else { (None, first) };

    expect_symbol(tokens, arena, ";")?;
    Ok(Statement::Assignment { destination, value })
}
fn expect_symbol<'b>(tokens: &mut Lexer, arena: &'b Arena, expected: &'static str) -> Result<(), ParseError<'b>> {
    let token = tokens.next()?;
    if token == expected {
        Ok(())
    }
    else {
        Err(ParseError::ExpectedSymbol { expected, found: arena.alloc_str(token.as_str()), line: arena.alloc_str(tokens.get_current_line()) })
    }
}
fn expect_keyword<'b>(tokens: &mut Lexer, arena: &'b Arena, expected: &'static str) -> Result<(), ParseError<'b>> {
    let token = tokens.next()?;
    if token == expected {
        Ok(())
    }
    else {
        Err(ParseError::ExpectedKeyword { expected, found: arena.alloc_str(token.as_str()), line: arena.alloc_str(tokens.get_current_line()) })
    }
}
fn next_identifier<'b>(tokens: &mut Lexer, arena: &'b Arena) -> Result<&'b [u8], ParseError<'b>> {
    let token = tokens.next()?;

    if !token.as_bytes()[0].is_ascii_alphabetic() {
        return Err(ParseError::ExpectedIdentifier { found: arena.alloc_str(token.as_str()), line: arena.alloc_str(tokens.get_current_line()) });
    }
    else {
        Ok(arena.alloc_slice(token.as_bytes()))
    }
}