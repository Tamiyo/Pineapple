use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::get_string;
use crate::parser::{binop::BinOp, relop::RelOp};
use std::cmp::Eq;
use std::{fmt, hash::Hash};

pub mod ssa;

type BlockIndex = usize;
type Label = usize;
type Interned = usize;
type Arity = usize;
type Sym = usize;
type Version = usize;

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Tac(Oper, Expr),
    Label(Label),
    NamedLabel(Label),
    Jump(Label),
    CJump(Expr, Label),
    Call(Interned, Arity),
    StackPush(Oper),
    //  Special pseudo-instruction for SSA destruction
    //  See SSA-Book p. 36
    ParallelCopy,
}

impl Stmt {
    pub fn contains_label(&self, label: usize) -> bool {
        match self {
            Stmt::Label(l) => *l == label,
            Stmt::Jump(j) => *j == label,
            Stmt::CJump(_, cj) => *cj == label,
            _ => false,
        }
    }

    pub fn replace_label(&mut self, label: usize) {
        match self {
            Stmt::Label(l) => *l = label,
            Stmt::Jump(j) => *j = label,
            Stmt::CJump(_, cj) => *cj = label,
            _ => (),
        }
    }

    pub fn def(&self) -> Vec<Oper> {
        match self {
            Stmt::Tac(lval, _) => match lval {
                Oper::Var(_, _) | Oper::Temp(_, _) => vec![*lval],
                _ => vec![],
            },
            _ => vec![],
        }
    }

    pub fn used(&self) -> Vec<Oper> {
        match self {
            Stmt::Tac(_, rval) => rval.oper_used(),
            Stmt::CJump(cond, _) => cond.oper_used(),
            Stmt::StackPush(oper) => vec![*oper],
            _ => vec![],
        }
    }

    pub fn replace_all_oper_def_with(&mut self, a: &Oper, b: &Oper) -> bool {
        match self {
            Stmt::Tac(lval, _) => lval.replace_oper_with(a, b),
            _ => false,
        }
    }

    pub fn replace_all_oper_use_with(&mut self, a: &Oper, b: &Oper) -> bool {
        match self {
            Stmt::Tac(_, rval) => rval.replace_oper_with(a, b),
            Stmt::CJump(cond, _) => {
                cond.replace_oper_with(a, b)
            }
            Stmt::StackPush(oper) => oper.replace_oper_with(a, b),
            _ => false,
        }
    }

    pub fn replace_oper_def_with_ssa(&mut self, value: Oper, ssa: usize) {
        if let Stmt::Tac(lval, _) = self {
            lval.replace_with_ssa(value, ssa)
        }
    }

    pub fn replace_oper_use_with_ssa(&mut self, value: Oper, ssa: usize) {
        match self {
            Stmt::Tac(_, rval) => rval.replace_with_ssa(value, ssa),
            Stmt::CJump(cond, _) => cond.replace_with_ssa(value, ssa),
            Stmt::StackPush(oper) => oper.replace_with_ssa(value, ssa),
            _ => (),
        }
    }

    pub fn patch_phi(&mut self, x: &Oper) -> bool {
        match self {
            Stmt::Tac(_, rval) => {
                if let Expr::Phi(args) = rval {
                    let mut i = 0;
                    while i < args.len() {
                        if args[i].0 == *x {
                            args.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                    if args.len() == 1 {
                        *rval = Expr::Oper(args[0].0);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Tac(lval, rval) => write!(f, "{:?} = {:?}", lval, rval),
            Stmt::Label(label) => write!(f, "_L{:?}:", label),
            Stmt::Jump(label) => write!(f, "goto _L{:?}", label),
            Stmt::CJump(cond, label) => write!(f, "if {:?} goto _L{:?}", cond, label),
            Stmt::ParallelCopy => write!(f, "? = ?"),
            Stmt::StackPush(rval) => write!(f, "_push {:?}", rval),
            Stmt::Call(sym, arity) => write!(f, "call {}({})", get_string(*sym), arity),
            _ => unimplemented!(),
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

impl Expr {
    pub fn oper_used(&self) -> Vec<Oper> {
        match self {
            Expr::Oper(o) => match *o {
                Oper::Var(_, _) => vec![*o],
                Oper::Temp(_, _) => vec![*o],
                _ => vec![],
            },
            Expr::Binary(l, _, r) => [
                match *l {
                    Oper::Var(_, _) => vec![*l],
                    Oper::Temp(_, _) => vec![*l],
                    _ => vec![],
                },
                match *r {
                    Oper::Var(_, _) => vec![*r],
                    Oper::Temp(_, _) => vec![*r],
                    _ => vec![],
                },
            ]
            .concat(),
            Expr::Logical(l, _, r) => [
                match *l {
                    Oper::Var(_, _) => vec![*l],
                    Oper::Temp(_, _) => vec![*l],
                    _ => vec![],
                },
                match *r {
                    Oper::Var(_, _) => vec![*r],
                    Oper::Temp(_, _) => vec![*r],
                    _ => vec![],
                },
            ]
            .concat(),
            Expr::Phi(args) => {
                let mut used: Vec<Oper> = vec![];
                for arg in args {
                    if let Oper::Var(_, _) = arg.0 {
                        used.push(arg.0)
                    }
                }
                used
            }
            _ => vec![],
        }
    }

    pub fn replace_oper_with(&mut self, a: &Oper, b: &Oper) -> bool {
        match self {
            Expr::Oper(o) => o.replace_oper_with(a, b),
            Expr::Binary(l, _, r) => {
                let a_b = l.replace_oper_with(a, b);
                let b_b = r.replace_oper_with(a, b);
                a_b || b_b
            }
            Expr::Logical(l, _, r) => {
                let a_b = l.replace_oper_with(a, b);
                let b_b = r.replace_oper_with(a, b);
                a_b || b_b
            }
            Expr::Phi(args) => {
                let mut res = false;
                for arg in args {
                    if arg.0 == *a {
                        arg.0 = *b;
                        res = true;
                    }
                }
                res
            }
            _ => false,
        }
    }

    pub fn replace_with_ssa(&mut self, value: Oper, ssa: usize) {
        match self {
            Expr::Oper(o) => o.replace_with_ssa(value, ssa),
            Expr::Binary(l, _, r) => {
                l.replace_with_ssa(value, ssa);
                r.replace_with_ssa(value, ssa);
            }
            Expr::Logical(l, _, r) => {
                l.replace_with_ssa(value, ssa);
                r.replace_with_ssa(value, ssa);
            }
            _ => (),
        }
    }
}

impl fmt::Debug for Expr {
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
                                write!(f, "{}.{}, ", get_string(value), ssa)?;
                            } else {
                                write!(f, "{}.{}", get_string(value), ssa)?;
                            }
                        }
                        Oper::Temp(value, ssa) => {
                            if i != args.len() - 1 {
                                write!(f, "_t{}.{}, ", value, ssa)?;
                            } else {
                                write!(f, "_t{}.{}", value, ssa)?;
                            }
                        }
                        Oper::Constant(c) => {
                            if i != args.len() - 1 {
                                write!(f, "{}, ", c)?;
                            } else {
                                write!(f, "{}", c)?;
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Oper {
    Var(Sym, Version),
    Temp(Sym, Version),
    Constant(Constant),

    Register(usize),
    StackLocation(usize),
}

impl Oper {
    pub fn as_non_ssa(&self) -> (usize, bool) {
        match self {
            Oper::Var(value, _) => (*value, true),
            Oper::Temp(value, _) => (*value, false),
            _ => panic!("Expected VAR or TEMP"),
        }
    }

    pub fn replace_oper_with(&mut self, a: &Oper, b: &Oper) -> bool {
        if self == a {
            *self = *b;
            true
        } else {
            false
        }
    }

    pub fn replace_with_ssa(&mut self, value: Oper, ssa: usize) {
        if let Oper::Var(v, s) = self {
            if let Oper::Var(a, _) = value {
                if *v == a {
                    *s = ssa;
                }
            }
        } else if let Oper::Temp(v, s) = self {
            if let Oper::Temp(a, _) = value {
                if *v == a {
                    *s = ssa;
                }
            }
        }
    }
}

impl fmt::Debug for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Oper::Register(reg) => write!(f, "${}", reg),
            Oper::StackLocation(offset) => write!(f, "sp[-{}]", offset),
            Oper::Var(value, ssa) => write!(f, "{}.{}", get_string(*value), ssa),
            Oper::Temp(value, ssa) => write!(f, "_t{}.{}", value, ssa),
            Oper::Constant(c) => write!(f, "{}", c),
        }
    }
}