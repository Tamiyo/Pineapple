pub mod cfg;

use crate::op::{BinOp, RelOp};
use crate::value::Value;
use crate::value::ValueTy;
use std::{cell::RefCell, rc::Rc};

type BlockIndex = usize;
type Interned = usize;
type Arity = usize;
type Sym = usize;
type Version = usize;

type Statement = Rc<RefCell<Stmt>>;

pub enum Label {
    Marker(usize),
    Named(usize),
}

pub enum Stmt {
    Tac(Oper, Expr),
    Label(Label),
    Jump(Label),
    CJump(Expr, Label),

    CastAs(Oper, ValueTy),

    Call(Interned, Arity),

    //  Ideally we want to remove these in favor of smarter
    //  function context pushing / popping, but we'll worry
    //  about that later.
    StackPushAllReg,
    StackPopAllReg,

    StackPush(Oper),

    Return(Option<Oper>),

    //  Special pseudo-instruction for SSA destruction
    //  See SSA-Book p. 36
    ParallelCopy(Vec<Statement>),
}

pub enum Expr {
    Binary(Oper, BinOp, Oper),
    Logical(Oper, RelOp, Oper),
    Oper(Oper),
    Phi(Vec<(Oper, BlockIndex)>),
}

pub enum Oper {
    Var(Sym, Version),
    Temp(Sym, Version),
    Value(Value),

    StackPop,

    Register(usize),
    ReturnValue,
    StackLocation(usize),
}
