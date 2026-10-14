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
        name: &'a str,
        parameters: &'b [Expression<'a, 'b>], 
    },
    Operator{
        operator: &'a str,
        left: &'b Expression<'a, 'b>,
        right: &'b Expression<'a, 'b>,
    },
    Field{
        name: &'a str,
        field: &'b Expression<'a, 'b>,
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
    
}
fn parse_statement<'a, 'b>(tokens: &mut Tokenizer<'a>, arena: &'b Arena) -> Option<Statement<'a, 'b>> {
    return;
}