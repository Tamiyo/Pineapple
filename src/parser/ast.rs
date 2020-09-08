use crate::core::value::Type;
use crate::core::{binop::BinOp, relop::RelOp, value::Value};

type Sym = usize;

type Condition = Box<Expr>;
type Body = Box<Stmt>;
type Args = Vec<(Sym, Type)>;

/**
 *  [AST Expression]
 *
 *  Defines an AST node for expression statements. These nodes contain
 *  heap-allocated pointers to other expressions.
 */
#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Variable(Sym),
    Assign(Sym, Type, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Logical(Box<Expr>, RelOp, Box<Expr>),
    Grouping(Box<Expr>),
    CastAs(Box<Expr>, Type),
    Call(Box<Expr>, Vec<Expr>),
}

/**
 *  [AST Statement]
 *
 *  Defines an AST node for statements. These nodes contain
 *  heap-allocated pointers to other statements or expressions.
 */
#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    If(Condition, Body, Option<Box<Stmt>>),
    While(Condition, Body),
    Expression(Box<Expr>),
    Print(Vec<Expr>),
    Return(Option<Box<Expr>>),
    Function(Sym, Args, Type, Vec<Stmt>),
}
