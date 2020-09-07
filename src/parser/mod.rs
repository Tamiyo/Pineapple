use crate::bytecode::string_intern::intern_string;
use crate::core::binop::BinOp;
use crate::core::relop::RelOp;
use crate::core::value::Primitive;
use crate::parser::ast::Expr;
use crate::parser::ast::Stmt;
use crate::parser::error::ParseError;
use crate::{
    bytecode::distancef64::DistanceF64,
    core::value::{Type, Value},
    parser::token::Token,
};
use std::cell::RefCell;
use token::Symbol;

pub mod ast;
pub mod error;
pub mod scanner;
pub mod token;

static mut TOKENS: RefCell<Vec<Token>> = RefCell::new(vec![]);

// Expression Parsing

#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assign,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Power,
    Unary,
    Index,
    Call,
}

impl From<Token> for Precedence {
    fn from(token: Token) -> Precedence {
        match *token {
            Symbol::Colon => Precedence::Assign,
            Symbol::Or => Precedence::Or,
            Symbol::And => Precedence::And,
            Symbol::NotEqual | Symbol::EqualEqual => Precedence::Equality,
            Symbol::Less | Symbol::LessEqual | Symbol::Greater | Symbol::GreaterEqual => {
                Precedence::Comparison
            }
            Symbol::Plus | Symbol::Minus => Precedence::Term,
            Symbol::Star | Symbol::Slash | Symbol::Modulo => Precedence::Factor,
            Symbol::Carat => Precedence::Power,
            Symbol::Not => Precedence::Unary,
            Symbol::LeftSquare => Precedence::Index,
            Symbol::LeftParen | Symbol::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

// Helpers
fn peek() -> Result<Token, ParseError> {
    unsafe {
        match TOKENS.borrow().last() {
            Some(token) => Ok(*token),
            None => Err(ParseError::TokenStreamEmpty),
        }
    }
}

fn next() -> Result<Token, ParseError> {
    unsafe {
        match TOKENS.borrow_mut().pop() {
            Some(token) => Ok(token),
            None => Err(ParseError::TokenStreamEmpty),
        }
    }
}

fn consume(expected: Symbol) -> Result<(), ParseError> {
    let found = peek()?;
    if *found != expected {
        Err(ParseError::UnexpectedToken(found.clone(), expected))
    } else {
        next()?;
        Ok(())
    }
}

fn consume_type() -> Result<Type, ParseError> {
    let found = next()?;

    match *found {
        Symbol::TypeInt8 => Ok(Type::Int8),
        Symbol::TypeInt16 => Ok(Type::Int16),
        Symbol::TypeInt32 => Ok(Type::Int32),
        Symbol::TypeInt64 => Ok(Type::Int64),
        Symbol::TypeInt => Ok(Type::Int),
        Symbol::TypeInt128 => Ok(Type::Int128),
        Symbol::TypeUInt8 => Ok(Type::UInt8),
        Symbol::TypeUInt16 => Ok(Type::UInt16),
        Symbol::TypeUInt32 => Ok(Type::UInt32),
        Symbol::TypeUInt64 => Ok(Type::UInt64),
        Symbol::TypeUInt => Ok(Type::UInt),
        Symbol::TypeUInt128 => Ok(Type::UInt128),
        Symbol::TypeFloat32 => Ok(Type::Float32),
        Symbol::TypeFloat64 => Ok(Type::Float64),
        Symbol::TypeBool => Ok(Type::Bool),
        _ => Err(ParseError::ExpectedVariableType(found)),
    }
}

pub fn parse_program(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParseError> {
    unsafe {
        let mut _t = tokens;
        _t.reverse();
        TOKENS.replace(_t);
    }

    let mut statements = vec![];
    while *peek()? != Symbol::Eof {
        statements.push(parse_declaration()?);
    }
    Ok(statements)
}

// Statement Parsing
fn parse_declaration() -> Result<Stmt, ParseError> {
    match *peek()? {
        Symbol::Fun => parse_function(),
        _ => parse_statement(),
    }
}

fn parse_function() -> Result<Stmt, ParseError> {
    consume(Symbol::Fun)?;
    let next = next()?;
    let name = match &*next {
        Symbol::Identifier(name) => Ok(name),
        _ => Err(ParseError::ExpectedIdentifier(peek()?.clone())),
    }?;

    consume(Symbol::LeftParen)?;
    let params: Vec<(usize, Type)> = if *peek()? != Symbol::RightParen {
        parse_identifier_list()?
    } else {
        Vec::new()
    };
    consume(Symbol::RightParen)?;

    let mut return_type = Type::None;
    if *peek()? == Symbol::Colon {
        consume(Symbol::Colon)?;
        return_type = consume_type()?;
    }

    consume(Symbol::LeftBrace)?;
    let mut statements: Vec<Stmt> = Vec::new();
    while *peek()? != Symbol::RightBrace && *peek()? != Symbol::Eof {
        statements.push(parse_declaration()?);
    }
    consume(Symbol::RightBrace)?;

    Ok(Stmt::Function(*name, params, return_type, statements))
}

fn parse_identifier_list() -> Result<Vec<(usize, Type)>, ParseError> {
    let mut identifiers: Vec<(usize, Type)> = Vec::new();
    let mut n = next()?;
    consume(Symbol::Colon)?;
    let ptype = consume_type()?;

    let sym = match &*n {
        Symbol::Identifier(name) => Ok(*name),
        _ => return Err(ParseError::ExpectedIdentifier(n.clone())),
    }?;
    identifiers.push((sym, ptype));

    while *peek()? == Symbol::Comma {
        consume(Symbol::Comma)?;
        n = next()?;
        consume(Symbol::Colon)?;
        let ptype = consume_type()?;

        let sym = match &*n {
            Symbol::Identifier(name) => Ok(*name),
            _ => return Err(ParseError::ExpectedIdentifier(n.clone())),
        }?;
        identifiers.push((sym, ptype));
    }
    Ok(identifiers)
}

fn parse_statement() -> Result<Stmt, ParseError> {
    match *peek()? {
        Symbol::If => parse_if_statement(),
        // Symbol::While => parse_while_statement(),
        Symbol::Print => parse_print_statement(),
        Symbol::Return => parse_return_statement(),
        _ => parse_expression_statement(),
    }
}

fn parse_block_statement() -> Result<Stmt, ParseError> {
    consume(Symbol::LeftBrace)?;

    let mut statements = Vec::new();
    while *peek()? != Symbol::RightBrace && *peek()? != Symbol::Eof {
        statements.push(parse_declaration()?);
    }

    consume(Symbol::RightBrace)?;
    Ok(Stmt::Block(statements))
}

fn parse_if_statement() -> Result<Stmt, ParseError> {
    consume(Symbol::If)?;
    consume(Symbol::LeftParen)?;
    let if_condition = parse_expression(Precedence::None, None)?;
    consume(Symbol::RightParen)?;

    let if_block = parse_block_statement()?;
    let rest = parse_elif_statement()?;

    Ok(Stmt::If(Box::new(if_condition), Box::new(if_block), rest))
}

fn parse_elif_statement() -> Result<Option<Box<Stmt>>, ParseError> {
    if *peek()? == Symbol::Elif {
        consume(Symbol::Elif)?;
        consume(Symbol::LeftParen)?;

        let elif_condition = parse_expression(Precedence::None, None)?;

        consume(Symbol::RightParen)?;
        let elif_block = parse_block_statement()?;
        Ok(Some(Box::new(Stmt::If(
            Box::new(elif_condition),
            Box::new(elif_block),
            parse_elif_statement()?,
        ))))
    } else {
        parse_else_statement()
    }
}

fn parse_else_statement() -> Result<Option<Box<Stmt>>, ParseError> {
    let else_block = if *peek()? == Symbol::Else {
        consume(Symbol::Else)?;
        Some(parse_block_statement()?)
    } else {
        Some(Stmt::Block(vec![]))
    };

    Ok(else_block.map(Box::new))
}

fn parse_print_statement() -> Result<Stmt, ParseError> {
    consume(Symbol::Print)?;
    consume(Symbol::LeftParen)?;
    let expr_list = parse_expression_list(None)?;
    consume(Symbol::RightParen)?;
    consume(Symbol::Semicolon)?;

    Ok(Stmt::Print(expr_list))
}

fn parse_return_statement() -> Result<Stmt, ParseError> {
    consume(Symbol::Return)?;
    let expr = if *peek()? != Symbol::Semicolon {
        Some(Box::new(parse_expression(Precedence::None, None)?))
    } else {
        None
    };
    consume(Symbol::Semicolon)?;
    Ok(Stmt::Return(expr))
}

fn parse_expression_list(expected_type: Option<Type>) -> Result<Vec<Expr>, ParseError> {
    let mut expressions = Vec::new();
    if *peek()? != Symbol::RightParen {
        expressions.push(parse_expression(Precedence::None, expected_type)?);
    }

    while *peek()? == Symbol::Comma {
        consume(Symbol::Comma)?;
        expressions.push(parse_expression(Precedence::None, expected_type)?);
    }

    Ok(expressions)
}

fn parse_expression_statement() -> Result<Stmt, ParseError> {
    let expr = parse_expression(Precedence::None, None)?;
    consume(Symbol::Semicolon)?;
    Ok(Stmt::Expression(Box::new(expr)))
}

// Expression Parsing
fn parse_expression(
    precedence: Precedence,
    expected_type: Option<Type>,
) -> Result<Expr, ParseError> {
    fn infix(left: &mut Expr, expected_type: Option<Type>) -> Result<Expr, ParseError> {
        match *peek()? {
            Symbol::Or
            | Symbol::And
            | Symbol::Plus
            | Symbol::Minus
            | Symbol::Star
            | Symbol::Slash
            | Symbol::Modulo
            | Symbol::Carat => parse_binary(left, expected_type),
            Symbol::Less
            | Symbol::LessEqual
            | Symbol::Greater
            | Symbol::GreaterEqual
            | Symbol::NotEqual
            | Symbol::EqualEqual => parse_logical(left, expected_type),
            Symbol::Colon => parse_assign(left),
            Symbol::LeftParen => parse_call(left, expected_type),
            _ => Err(ParseError::UnexpectedInfixOperator(peek()?)),
        }
    }

    fn prefix(expected_type: Option<Type>) -> Result<Expr, ParseError> {
        match *peek()? {
            Symbol::IntegerLiteral(_)
            | Symbol::FloatLiteral(_)
            | Symbol::Identifier(_)
            | Symbol::StringLiteral(_) => parse_primary(expected_type),
            Symbol::True => {
                next()?;
                Ok(Expr::Value(Value::from(Primitive::Bool(true))))
            }
            Symbol::False => {
                next()?;
                Ok(Expr::Value(Value::from(Primitive::Bool(false))))
            }
            Symbol::LeftParen => parse_grouping(expected_type),
            _ => Err(ParseError::UnexpectedPrefixOperator(peek()?)),
        }
    }

    let mut expr = prefix(expected_type)?;
    while let Ok(token) = peek() {
        let next = Precedence::from(token);
        if precedence >= next {
            break;
        }
        expr = infix(&mut expr, expected_type)?;
    }
    Ok(expr)
}

fn parse_assign(left: &mut Expr) -> Result<Expr, ParseError> {
    consume(Symbol::Colon)?;

    let expected_type = consume_type()?;

    consume(Symbol::Equal)?;
    let right = parse_expression(Precedence::None, Some(expected_type))?;
    match left {
        Expr::Variable(identifier) => Ok(Expr::Assign(*identifier, expected_type, Box::new(right))),
        _ => Err(ParseError::ExpectedLValue(left.clone())),
    }
}

fn parse_call(left: &mut Expr, expected_type: Option<Type>) -> Result<Expr, ParseError> {
    consume(Symbol::LeftParen)?;
    let args = parse_expression_list(expected_type)?;
    consume(Symbol::RightParen)?;
    Ok(Expr::Call(Box::new(left.clone()), args))
}

fn parse_grouping(expected_type: Option<Type>) -> Result<Expr, ParseError> {
    consume(Symbol::LeftParen)?;
    let expr = parse_expression(Precedence::None, expected_type)?;
    consume(Symbol::RightParen)?;
    Ok(Expr::Grouping(Box::new(expr)))
}

fn parse_binary(left: &mut Expr, expected_type: Option<Type>) -> Result<Expr, ParseError> {
    let precedence = Precedence::from(peek()?);

    let op = next()?;
    match *op {
        Symbol::Or
        | Symbol::And
        | Symbol::Plus
        | Symbol::Minus
        | Symbol::Star
        | Symbol::Slash
        | Symbol::Modulo
        | Symbol::Carat => Ok(()),
        _ => {
            return Err(ParseError::ExpectedBinaryOperator(op));
        }
    }?;

    let right = parse_expression(precedence, expected_type)?;

    Ok(Expr::Binary(
        Box::new(left.clone()),
        BinOp::from(&op),
        Box::new(right),
    ))
}

fn parse_logical(left: &mut Expr, expected_type: Option<Type>) -> Result<Expr, ParseError> {
    let precedence = Precedence::from(peek()?);

    let op = next()?;
    match *op {
        Symbol::Less
        | Symbol::LessEqual
        | Symbol::Greater
        | Symbol::GreaterEqual
        | Symbol::NotEqual
        | Symbol::EqualEqual => Ok(()),
        _ => {
            return Err(ParseError::ExpectedBinaryOperator(op));
        }
    }?;

    let right = parse_expression(precedence, expected_type)?;

    Ok(Expr::Logical(
        Box::new(left.clone()),
        RelOp::from(&op),
        Box::new(right),
    ))
}

fn parse_primary(expected_type: Option<Type>) -> Result<Expr, ParseError> {
    let token = next()?;
    if let Some(expected_type) = expected_type {
        match token.sym {
            Symbol::IntegerLiteral(number) => {
                if let Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::Int
                | Type::Int128 = expected_type
                {
                    let number: i128 = number as i128;
                    let prim = match Primitive::Int128(number).try_cast_to(expected_type) {
                        Ok(p) => p,
                        Err(()) => return Err(ParseError::CastError(token, expected_type)),
                    };
                    return Ok(Expr::Value(Value::from(prim)));
                }

                if let Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::UInt
                | Type::UInt128 = expected_type
                {
                    let number: u128 = number as u128;
                    let prim = match Primitive::UInt128(number).try_cast_to(expected_type) {
                        Ok(p) => p,
                        Err(()) => return Err(ParseError::CastError(token, expected_type)),
                    };
                    return Ok(Expr::Value(Value::from(prim)));
                }
            }
            Symbol::FloatLiteral(number) => {
                let prim = match Primitive::Float64(DistanceF64::from(number as f64))
                    .try_cast_to(expected_type)
                {
                    Ok(p) => p,
                    Err(()) => return Err(ParseError::CastError(token, expected_type)),
                };
                return Ok(Expr::Value(Value::from(prim)));
            }
            Symbol::Identifier(sym) => {
                return Ok(Expr::Variable(sym));
            }
            _ => return Err(ParseError::ExpectedLiteral(token)),
        }
    } else {
        match token.sym {
            Symbol::IntegerLiteral(number) => {
                return Ok(Expr::Value(Value::from(Primitive::UInt(number as usize))));
            }
            Symbol::FloatLiteral(number) => {
                return Ok(Expr::Value(Value::from(Primitive::Float64(
                    DistanceF64::from(number as f64),
                ))));
            }
            Symbol::Identifier(sym) => {
                return Ok(Expr::Variable(sym));
            }
            _ => return Err(ParseError::ExpectedLiteral(token)),
        }
    }

    Err(ParseError::ExpectedLiteral(token))
}
