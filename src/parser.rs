#![allow(unused)]

use std::ops::DerefMut;

use crate::{arena::Arena, lexer::Tokenizer};

// enum, switch+case+default, string literal, better int/float literals

#[derive(Clone, PartialEq, Debug)]
pub enum Expression<'a, 'b> {
    Integer(i64),
    Float(f64),
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
pub struct TypeAlias<'a, 'b> {
    public: bool,
    name: &'a [u8],
    typ: Type<'a, 'b>,
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
    value: Expression<'a, 'b>,
}
#[derive(Debug)]
pub enum Fragment<'a, 'b> {
    ASTfunction(ASTfunction<'a, 'b>),
    Include(Include<'a>),
    Constant(Constant<'a, 'b>),
    TypeAlias(TypeAlias<'a, 'b>),
    Struct(Struct<'a, 'b>),
    Union(Union<'a, 'b>),
    GlobalVariable(GlobalVariable<'a, 'b>),
}
pub struct Fragmenter<'a> {
    tokenizer: Tokenizer<'a>,
}
impl<'a> Fragmenter<'a> {
    pub fn next<'b>(&mut self, arena: &'b Arena) -> Option<Fragment<'a, 'b>> {
        parse_file(&mut self.tokenizer, &arena)
    }
    pub fn new(tokenizer: Tokenizer) -> Fragmenter {
        Fragmenter { tokenizer }
    }
}
fn parse_file<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Fragment<'a, 'b>> {
    if tokens.peek().is_none() {
        return (None);
    }

    if tokens.peek()? == "include" {
        return Some(Fragment::Include(parse_include(tokens)?));
    }

    let public = if tokens.peek()? == "public" {
        tokens.next();
        true
    }
    else {
        false
    };  

    if tokens.peek()? == "const" {
        return Some(Fragment::Constant(parse_constant(tokens, arena, public)?));
    }
    if tokens.peek()? == "struct" {
        return Some(Fragment::Struct(parse_struct(tokens, arena, public)?));
    }
    if tokens.peek()? == "union" {
        return Some(Fragment::Union(parse_union(tokens, arena, public)?));
    }
    if tokens.peek()? == "type" {
        return Some(Fragment::TypeAlias(parse_alias(tokens, arena, public)?));
    }
    if tokens.peek()? == "static" {
        return Some(Fragment::GlobalVariable(parse_global_var(tokens, arena, public)?));
    }
    return Some(Fragment::ASTfunction(parse_function(tokens, arena, public)?));
}
fn parse_global_var<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Option<GlobalVariable<'a, 'b>> {
    if tokens.next()? != "static" { return None; }

    let name = tokens.next()?;

    if tokens.next()? != ":" { return None; }

    let t = parse_type(tokens, arena)?;

    if tokens.next()? != "=" { return None; }

    let value = parse_expression(tokens, arena)?;

    if tokens.next()? != ";" { return None; }

    return Some(GlobalVariable { public, name: name.as_bytes(), typ: t, value });
}
fn parse_include<'a>(tokens: &mut Tokenizer<'a>) -> Option<Include<'a>> {
    if tokens.next()? != "include" { return None; }
    
    let path = tokens.until_semicolon()?;

    Some(Include{ path: path.as_bytes() })
}
fn parse_function<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Option<ASTfunction<'a, 'b>> {
    let return_type = if tokens.peek()? == "void" {
        None
    }
    else {
        Some(parse_type(tokens, &arena)?)
    };

    let name = tokens.next()?;

    if tokens.next()? != "(" { return None; }

    let mut parameters = arena.vec_capacity(16);

    while tokens.peek().is_some_and(|x| x != ")") {

        if tokens.peek()? == "," { tokens.next(); }

        let t = parse_type(tokens, &arena)?;
        let name = tokens.next()?.as_bytes();

        parameters.push((t, name));
    }

    if tokens.next()? != ")" { return None; }
    
    let body = parse_body(tokens, arena)?;

    return Some(ASTfunction{ 
        public,
        return_type, 
        name: name.as_bytes(), 
        parameters: parameters.array(), 
        body
    });
}
fn parse_type<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Type<'a, 'b>> {
    let first = tokens.next()?;
    if first == "*" {
        return Some(Type::Pointer(arena.alloc(parse_type(tokens, arena)?)));
    }
    if first == "[" {
        let t = arena.alloc(parse_type(tokens, arena)?);
        
        if tokens.next()? != ";" { return None; }

        let len = parse_expression(tokens, arena)?;

        if tokens.next()? != "]" { return None; }

        return Some(Type::Array(t, len));
    }
    return Some(Type::Name(first.as_bytes()));
}
fn parse_expression<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Expression<'a, 'b>> {
    let mut bracket_depth = 0;
    let e = parse_binary_expression(tokens, arena, 0, &mut bracket_depth)?;
    if bracket_depth != 0 {
        return None;
    }
    Some(e)
}
fn parse_binary_expression<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, min_precedence: i8, bracket_depth: &mut usize) -> Option<Expression<'a, 'b>> {

    if b"!*&-~".contains(&tokens.peek()?.as_bytes()[0]) { 
        let prefix = tokens.next()?;
        let e = parse_binary_expression(tokens, arena, min_precedence, bracket_depth)?;
        return Some(Expression::UnaryOperator { operator: token_to_unary_operator(tokens.next()?.as_bytes())?, expr: arena.alloc(e) });
    }

    let mut left = if tokens.peek()? == "(" {
        tokens.next();
        *bracket_depth += 1;
        parse_binary_expression(tokens, arena, 0, bracket_depth)?
    }
    else {
        parse_primary(tokens, arena)?
    };

    while let Some(op_token) = tokens.peek() {
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
                operator: token_to_binary_operator(operator.as_bytes())?,
                left: arena.alloc(left),
                right: arena.alloc(e),
            };
        }
    }

    if tokens.peek()? == ")" {
        if *bracket_depth == 0 {
            return None;
        }
        tokens.next();
        *bracket_depth -= 1;
        return Some(left);
    }

    Some(left)
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
fn parse_primary<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Expression<'a, 'b>> {
    let mut token = tokens.peek()?;

    if token == "[" {
        tokens.next();
        let mut parameters = arena.vec_capacity(16);
        

        while tokens.peek()? != "]" {

            if tokens.peek()? == "," { tokens.next(); }

            let expr = parse_expression(tokens, &arena)?;

            parameters.push(expr);
        }
        if tokens.next()? != "]" { return None; }

        return Some(Expression::ArrayInit(parameters.array()));
    }
    if token == "{" {
        tokens.next();
        let mut fields = arena.vec_capacity(16);

        while tokens.peek()? != "}" {

            if tokens.peek()? == "," { tokens.next(); }

            let name = tokens.next()?;

            if tokens.next()? != ":" { return None; }

            let value = parse_expression(tokens, &arena)?;

            fields.push((name.as_bytes(), value));
        }
        if tokens.next()? != "}" { return None; }

        return Some(Expression::StructInit(fields.array()));
    }

    if let Ok(val) = str::from_utf8(token.as_bytes()).unwrap().parse::<i64>() {
        let checkpoint = tokens.checkpoint();
        tokens.next();

        if tokens.peek()? == "." {
            tokens.restore(checkpoint);

            if let Ok(valf) = str::from_utf8(tokens.combine(3)?.as_bytes()).unwrap().parse::<f64>() {
                return Some(Expression::Float(valf));
            }
        }
        else {
            return Some(Expression::Integer(val));
        }
    }
    
    if token == "true" {
        tokens.next();
        return Some(Expression::Bool(true));
    }
    
    if token == "false" {
        tokens.next();
        return Some(Expression::Bool(false));
    }

    tokens.next();
    let mut expr = Expression::Name(token.as_bytes());

    loop {
        if tokens.peek()? == "(" {
            tokens.next();
            let mut parameters = arena.vec_capacity(16);

            while tokens.peek().is_some_and(|x| x != ")") {
                if tokens.peek()? == "," {
                    tokens.next();
                }

                let arg = parse_expression(tokens, arena)?;
                parameters.push(arg);
            }

            if tokens.next()? != ")" { return None; }
 
            expr = Expression::Call {
                expr: arena.alloc(expr),
                parameters: parameters.array(),
            };
        } 
        else if tokens.peek()? == "." {
            tokens.next();
            let field_name = tokens.next()?;

            expr = Expression::Field {
                expr: arena.alloc(expr),
                field: field_name.as_bytes(),
            };
        }
        else if tokens.peek()? == "[" {
            tokens.next();

            let index = parse_expression(tokens, arena)?;

            if tokens.next()? != "]" { return None; }

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
    Some(expr)
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
fn parse_body<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<&'b [Statement<'a, 'b>]> {
    if tokens.next()? != "{" { return None; }

    let mut statements = arena.vec_capacity(64);

    while tokens.peek().is_some_and(|x| x != "}") {
        statements.push(parse_statement(tokens, &arena)?);
    }

    if tokens.next()? != "}" { return None; }

    Some(statements.array())
}
fn parse_statement<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Statement<'a, 'b>> {

    if tokens.peek()? == "return" {
        tokens.next();

        let value = if tokens.peek()? != ";" {
            Some(parse_expression(tokens, arena)?)
        }
        else {
            None
        };
        tokens.next();

        return Some(Statement::Return { value });
    }
    if tokens.peek()? == "break" {
        tokens.next();
        if tokens.next()? != ";" { return None; }
        return Some(Statement::Break);
    }
    if tokens.peek()? == "continue" {
        tokens.next();
        if tokens.next()? != ";" { return None; }
        return Some(Statement::Continue);
    }
    if tokens.peek()? == "while" {
        tokens.next();
        
        let condition = parse_expression(tokens, arena)?;

        let body = parse_body(tokens, arena)?;

        return Some(Statement::While { condition, body });
    }
    if tokens.peek()? == "if" {
        tokens.next();
        
        let condition = parse_expression(tokens, arena)?;

        let body = parse_body(tokens, arena)?;

        let else_body = if tokens.peek()? == "else" {
            tokens.next();
            Some(parse_body(tokens, arena)?)
        }
        else {
            None
        };

        return Some(Statement::If { condition, body, else_body });
    }
    if tokens.peek()? == "let" {
        tokens.next();
        
        let name = tokens.next()?;

        if tokens.next().unwrap() != ":" { panic!(); }

        let t = parse_type(tokens, arena)?;

        let value = if tokens.peek()? == "=" {
            tokens.next();
            Some(parse_expression(tokens, arena)?)
        }
        else {
            None
        };

        if tokens.next()? != ";" { return None; }

        return Some(Statement::Declaration { typ: t, name: name.as_bytes(), value });
    }
    if tokens.peek()? == "for" {
        tokens.next();

        if tokens.next()? != "(" { return None; }

        let declaration = if tokens.peek()? != ";" {
            if tokens.next()? != "let" { return None; }

            let name = tokens.next()?;

            let t = parse_type(tokens, arena)?;

            let value = if tokens.peek()? == "=" {
                tokens.next();
                Some(parse_expression(tokens, arena)?)
            }
            else {
                None
            };            

            let tmp: &Statement<'_, '_> = arena.alloc(Statement::Declaration { typ: t, name: name.as_bytes(), value });
            Some(tmp)
        }
        else {
            None
        };

        if tokens.next()? != ";" { return None; }

        let condition = if tokens.peek()? != ";" {
            Some(parse_expression(tokens, arena)?)
        }
        else {
            None
        };

        if tokens.next()? != ";" { return None; }

        let iteration = if tokens.peek()? != ";" {
            let first = parse_expression(tokens, arena)?;

            let second = if tokens.peek()?.as_bytes().last()? == &b'=' {
                let op = tokens.next()?;
                Some((parse_expression(tokens, arena)?, op))
            }
            else {
                None
            };

            if let Some((expr, op)) = second {
                if op == "=" {
                    Some(Statement::Assignment { destination: Some(first), value: expr })
                }
                else {
                    let operator = token_to_binary_operator(&op.as_bytes()[..op.as_bytes().len()-2])?;
                    let new_expr = Expression::BinaryOperator { operator, left: arena.alloc(first.clone()), right: arena.alloc(expr) };
                    Some(Statement::Assignment { destination: Some(first), value: new_expr })
                }
            }
            else {
                Some(Statement::Assignment { destination: None, value: first })
            }
        }
        else {
            None
        };

        if tokens.next()? != ";" { return None; }

        let iteration = if let Some(s) = iteration {
            let a: &Statement<'_, '_> = arena.alloc(s);
            Some(a)
        }
        else { None };

        if tokens.next()? != ")" { return None; }

        let body = parse_body(tokens, arena)?;

        return Some(Statement::For { declaration, condition, iteration, body });
    }
    if tokens.peek()? == "do" {
        tokens.next()?;
        
        let body = parse_body(tokens, arena)?;

        if tokens.next()? != "while" { return None; }

        let condition = parse_expression(tokens, arena)?;

        if tokens.next()? != ";" { return None; }

        return Some(Statement::DoWhile { body, condition })
    }

    let first = parse_expression(tokens, arena)?;

    let second = if tokens.peek()?.as_bytes().last()? == &b'=' {
        let op = tokens.next()?;
        Some((parse_expression(tokens, arena)?, op))
    }
    else {
        None
    };

    if tokens.next()? != ";" { return None; }

    if let Some((expr, op)) = second {

        if op == "=" {
            return Some(Statement::Assignment { destination: Some(first), value: expr });
        }
        else {
            let operator = token_to_binary_operator(&op.as_bytes()[..op.as_bytes().len()-2])?;
            let new_expr = Expression::BinaryOperator { operator, left: arena.alloc(first.clone()), right: arena.alloc(expr) };
            return Some(Statement::Assignment { destination: Some(first), value: new_expr });
        }
    }
    else {
        return Some(Statement::Assignment { destination: None, value: first });
    }
}
fn parse_struct<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Option<Struct<'a, 'b>> {
    if tokens.next()? != "struct" { return None; }

    let name = tokens.next()?;

    if tokens.next()? != "{" { return None; }

    let mut fields = arena.vec_capacity(16);

    while tokens.peek()? != "}" {
        if tokens.peek()? == "," {
            tokens.next();
        }

        let t = parse_type(tokens, arena)?;
        let name = tokens.next()?;
            
        fields.push((t, name.as_bytes()));
    }

    if tokens.next()? != "}" { return None; }

    return Some(Struct { public, name: name.as_bytes(), fields: fields.array() });

}
fn parse_union<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Option<Union<'a, 'b>> {
    if tokens.next()? != "union" { return None; }

    let name = tokens.next()?;

    if tokens.next()? != "{" { return None; }

    let mut fields = arena.vec_capacity(16);

    while tokens.peek()? != "}" {
        if tokens.peek()? == "," {
            tokens.next();
        }

        let t = parse_type(tokens, arena)?;
        let name = tokens.next()?;
        
        fields.push((t, name.as_bytes()));
    }

    if tokens.next()? != "}" { return None; }

    return Some(Union { public, name: name.as_bytes(), fields: fields.array() });

}
fn parse_alias<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Option<TypeAlias<'a, 'b>> {
    if tokens.next()? != "type" { return None; }

    let name = tokens.next()?;

    if tokens.next()? != "=" { return None; }

    let t = parse_type(tokens, arena)?;
    
    return Some(TypeAlias { public, name: name.as_bytes(), typ: t })
}
fn parse_constant<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, public: bool) -> Option<Constant<'a, 'b>> {
    if tokens.next()? != "const" { return None; }

    let name = tokens.next()?;

    if tokens.next()? != ":" { return None; }

    let t = parse_type(tokens, arena)?;

    if tokens.next()? != "=" { return None; }

    let value = parse_expression(tokens, arena)?;
    
    return Some(Constant { public, name: name.as_bytes(), typ: t, value });
}