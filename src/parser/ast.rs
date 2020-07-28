use crate::bytecode::distance::Distance;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;

type Interned = usize;

/**
 *  [AST Expression]
 *
 *  Defines an AST node for expression statements. These nodes contain
 *  heap-allocated pointers to other expressions.
 */
#[derive(Debug, Clone)]
pub enum Expr {
    Number(Distance),
    Boolean(bool),
    Variable(Interned),
    Assign(Interned, Box<Expr>),
    // Call(Box<Expr>, Vec<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Logical(Box<Expr>, RelOp, Box<Expr>),
    Grouping(Box<Expr>),
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
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    Expression(Box<Expr>),
    Print(Vec<Expr>),
    Return(Option<Box<Expr>>),
    Function(Interned, Vec<Interned>, Vec<Stmt>),
}
