use pineapple_ir::op::{BinOp, RelOp};
use pineapple_ir::value::{Value, ValueTy};

type Identifier = usize;
type Type = ValueTy;

type Condition = Box<Expr>;
type Body = Box<Stmt>;
type Args = Vec<(Identifier, Type)>;
type ReturnType = Type;

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Variable(Identifier),
    Assign(Identifier, Option<Type>, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Logical(Box<Expr>, RelOp, Box<Expr>),
    Grouping(Box<Expr>),
    CastAs(Box<Expr>, Type),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    If(Condition, Body, Option<Box<Stmt>>),
    While(Condition, Body),
    Expression(Box<Expr>),
    Print(Vec<Expr>),
    Return(Option<Box<Expr>>),
    Function(Identifier, Args, ReturnType, Box<Stmt>),
}
