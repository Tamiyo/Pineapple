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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Label {
    Marker(usize),
    Named(usize),
}

#[derive(Clone, PartialEq)]
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

impl std::fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Tac(lval, rval) => write!(f, "{:?} = {:?}", lval, rval),
            Stmt::Label(label) => match label {
                Label::Marker(label) => write!(f, "_L{}:", label),
                Label::Named(label)  => write!(f, "_{}:", label),
            },
            Stmt::CastAs(oper, t) => write!(f, "cast {:?} as {:?}", oper, t),
            Stmt::Jump(label) => write!(f, "goto _L{:?}", label),
            Stmt::CJump(cond, label) => write!(f, "if {:?} goto _L{:?}", cond, label),
            Stmt::ParallelCopy(copies) => {
                for copy in copies {
                    write!(f, "(p) {:?}", *copy.borrow())?;
                }
                Ok(())
            }
            Stmt::StackPushAllReg => write!(f, "_push all"),
            Stmt::StackPopAllReg => write!(f, "_pop all"),
            Stmt::StackPush(rval) => write!(f, "_push {:?}", rval),
            Stmt::Call(sym, arity) => write!(f, "call {}({})", sym, arity),
            Stmt::Return(oper) => write!(f, "ret {:?}", oper),
        }
    }
}


#[derive(Clone, PartialEq)]
pub enum Expr {
    Binary(Oper, BinOp, Oper),
    Logical(Oper, RelOp, Oper),
    Oper(Oper),
    Phi(Vec<(Oper, BlockIndex)>),
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::Binary(l, o, r) => write!(f, "{:?} {:?} {:?}", *l, o, *r),
            Expr::Logical(l, o, r) => write!(f, "{:?} {:?} {:?}", *l, o, *r),
            Expr::Oper(o) => write!(f, "{:?}", *o),
            Expr::Phi(args) => {
                write!(f, "Φ(")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "B{}: ", arg.1)?;
                    match arg.0 {
                        Oper::Var(value, ssa) => {
                            if i != args.len() - 1 {
                                write!(f, "{}.{}, ", value, ssa)?;
                            } else {
                                write!(f, "{}.{}", value, ssa)?;
                            }
                        }
                        Oper::Temp(value, ssa) => {
                            if i != args.len() - 1 {
                                write!(f, "_t{}.{}, ", value, ssa)?;
                            } else {
                                write!(f, "_t{}.{}", value, ssa)?;
                            }
                        }
                        Oper::Value(c) => {
                            if i != args.len() - 1 {
                                write!(f, "{:?}, ", c)?;
                            } else {
                                write!(f, "{:?}", c)?;
                            }
                        }
                        _ => (),
                    }
                }
                write!(f, ")")
            }
            _ => unimplemented!(),
        }
    }
}


#[derive(Copy, Clone, PartialEq)]
pub enum Oper {
    Var(Sym, Version),
    Temp(Sym, Version),
    Value(Value),

    StackPop,

    Register(usize),
    ReturnValue,
    StackLocation(usize),
}

impl std::fmt::Debug for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Oper::Register(reg) => write!(f, "${}", reg),
            Oper::StackLocation(offset) => write!(f, "sp[-{}]", offset),
            Oper::StackPop => write!(f, "_pop"),
            Oper::ReturnValue => write!(f, "$rv"),
            Oper::Var(value, ssa) => write!(f, "{}.{}", value, ssa),
            Oper::Temp(value, ssa) => write!(f, "_t{}.{}", value, ssa),
            Oper::Value(c) => write!(f, "{:?}", c),
        }
    }
}
