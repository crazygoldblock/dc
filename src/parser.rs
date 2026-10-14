use crate::{arena::Arena, tokenizer::Tokenizer};

struct Constant<'a, 'b> {
    public: bool,
    name: &'a str,
    typ: Type<'a, 'b>,
    value: Expression<'a, 'b>,
}
struct TypeAlias<'a, 'b> {
    public: bool,
    name: &'a str,
    typ: Type<'a, 'b>,
}
struct Struct<'a, 'b> {
    public: bool,
    name: &'a str,
    fields: &'b [(Type<'a, 'b>, &'a str)],
}
struct Function<'a, 'b> {
    public: bool,
    return_type: Type<'a, 'b>,
    name: &'a str,
    parameters: &'b [(Type<'a, 'b>, &'a str)],
    body: &'b [Statement<'a, 'b>],
}
#[derive(Clone)]
enum Statement<'a, 'b> {
    Assignment{
        destination: Option<Expression<'a, 'b>>,
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
        name: &'a str,
        value: Option<Expression<'a, 'b>>,
    },
    // for, which, case, default
}
#[derive(Clone)]
enum Expression<'a, 'b> {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Name(&'a str),
    Call{
        name: &'b Expression<'a, 'b>,
        parameters: &'b [Expression<'a, 'b>], 
    },
    BinaryOperator{
        operator: &'a str,
        left: &'b Expression<'a, 'b>,
        right: &'b Expression<'a, 'b>,
    },
    UnaryOperator{
        operator: &'a str,
        exp: &'b Expression<'a, 'b>,
    },
    Field{
        name: &'b Expression<'a, 'b>,
        field: &'a str,
    },
}
#[derive(Clone)]
enum Type<'a, 'b> {
    Name(&'a str),
    Pointer(&'b Type<'a, 'b>),
    Array(&'b Type<'a, 'b>, Expression<'a, 'b>),
}


pub fn parse_function<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Function<'a, 'b>> {
    let first = tokens.peek()?;
    let public = if first == "public" {
        tokens.next();
        true
    }
    else {
        false
    };

    let return_type = parse_type(tokens, &arena)?;

    let name = tokens.next()?;

    if tokens.next()? != "(" { return None; }

    let parameters = arena.alloc_array((Type::Name(""), ""), 64);
    let mut parameters_count = 0;

    while tokens.peek().is_some_and(|x| x != ")") {

        if tokens.peek()? == "," { tokens.next(); }

        let t = parse_type(tokens, &arena)?;
        let name = tokens.next()?;

        parameters[parameters_count] = (t, name);
        parameters_count += 1;
    }

    if tokens.next()? != ")" { return None; }
    if tokens.next()? != "{" { return None; }

    let statements = arena.alloc_array(Statement::Break, 256);
    let mut statements_count = 0;

    while tokens.peek().is_some_and(|x| x != "}") {

        let s = parse_statement(tokens, &arena)?;

        statements[statements_count] = s;
        statements_count += 1;
    }

    if tokens.next()? != "}" { return None; }

    return Some(Function{ 
        public,
        return_type, 
        name, 
        parameters: &parameters[..parameters_count], 
        body: &statements[..statements_count] 
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
    return Some(Type::Name(first));
}
fn parse_expression<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Expression<'a, 'b>> {
    parse_binary_expression(tokens, arena, 0)
}
fn parse_binary_expression<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena, min_precedence: u8) -> Option<Expression<'a, 'b>> {
    let mut left = parse_primary(tokens, arena)?;

    while let Some(op_token) = tokens.peek() {
        let precedence = get_operator_precedence(op_token);
        if precedence < min_precedence {
            break;
        }
        let operator = tokens.next()?;
        
        let right = parse_binary_expression(tokens, arena, precedence + 1)?;

        left = Expression::BinaryOperator {
            operator,
            left: arena.alloc(left),
            right: arena.alloc(right),
        };
    }
    Some(left)
}

fn parse_primary<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Expression<'a, 'b>> {
    let mut token = tokens.next()?;

    let prefix = if "!*&-~".contains(token) {
        token = tokens.next()?;
        Some(token)
    }
    else { None };

    if let Ok(val) = token.parse::<i64>() {
        return Some(Expression::Integer(val));
    }

    if let Ok(val) = token.parse::<f64>() {
        return Some(Expression::Float(val));
    }

    if token == "true" {
        return Some(Expression::Bool(true));
    }
    if token == "false" {
        return Some(Expression::Bool(false));
    }

    let mut expr = Expression::Name(token);

    loop {
        if tokens.peek() == Some("(") {
            tokens.next();
            let parameters = arena.alloc_array(Expression::Integer(0), 64);
            let mut parameters_count = 0;

            while tokens.peek().is_some_and(|x| x != ")") {
                if tokens.peek()? == "," {
                    tokens.next();
                }

                let arg = parse_expression(tokens, arena)?;
                parameters[parameters_count] = arg;
                parameters_count += 1;
            }

            if tokens.next()? != ")" { return None; }
 
            expr = Expression::Call {
                name: arena.alloc(expr),
                parameters: &parameters[..parameters_count],
            };
        } 
        else if tokens.peek() == Some(".") {
            tokens.next();
            let field_name = tokens.next()?;

            expr = Expression::Field {
                name: arena.alloc(expr),
                field: field_name,
            };
        }
        else if tokens.peek() == Some("[") {
            tokens.next();

            let index = parse_expression(tokens, arena)?;

            if tokens.next()? != "]" { return None; }

            expr = Expression::BinaryOperator { 
                operator: "[", 
                left: arena.alloc(expr), 
                right: arena.alloc(index) 
            };    
        }
        else {
            break;
        }
    }

    if let Some(op) = prefix {
        expr = Expression::UnaryOperator { operator: op, exp: arena.alloc(expr) }
    }
    Some(expr)
}
fn get_operator_precedence(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "|" => 3,
        "^" => 4,
        "&" => 5,
        "==" | "!=" => 6,
        "<" | ">" | "<=" | ">=" => 7,
        "<<" | ">>" => 8,
        "+" | "-" => 9,
        "*" | "/" | "%" => 10,
        _ => 0,
    }
}
fn parse_statement<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Statement<'a, 'b>> {
    return;
}