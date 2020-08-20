use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::get_string;
use crate::parser::{binop::BinOp, relop::RelOp};
use std::cmp::Eq;
use std::{fmt, hash::Hash};

type Label = usize;
type Identifier = usize;
type SSA = usize;

#[derive(Clone)]
pub enum Stmt {
    Tac(Oper, Expr),
    Label(Label),
    Jump(Label),
    CJump(Expr, Label),

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
            _ => vec![],
        }
    }

    pub fn replace_oper_def_with_ssa(&mut self, value: Oper, ssa: usize) {
        match self {
            Stmt::Tac(lval, _) => lval.replace_with_ssa(value, ssa),
            _ => (),
        }
    }

    pub fn replace_oper_use_with_ssa(&mut self, value: Oper, ssa: usize) {
        match self {
            Stmt::Tac(_, rval) => rval.replace_with_ssa(value, ssa),
            Stmt::CJump(cond, _) => cond.replace_with_ssa(value, ssa),
            _ => (),
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Tac(lval, rval) => write!(f, "\t{:?} = {:?}", lval, rval),
            Stmt::Label(label) => write!(f, "_L{:?}:", label),
            Stmt::Jump(label) => write!(f, "\tgoto _L{:?}", label),
            Stmt::CJump(cond, label) => write!(f, "\tif {:?} goto _L{:?}", cond, label),
            Stmt::ParallelCopy => write!(f, "\t? = ?"),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Binary(Oper, BinOp, Oper),
    Logical(Oper, RelOp, Oper),
    Oper(Oper),
    Phi(Vec<Oper>),
}

impl Expr {
    pub fn oper_used(&self) -> Vec<Oper> {
        match self {
            Expr::Oper(o) => match *o {
                Oper::Var(_, _) | Oper::Temp(_, _) => vec![*o],
                _ => vec![],
            },
            Expr::Binary(l, _, r) => [
                match *l {
                    Oper::Var(_, _) | Oper::Temp(_, _) => vec![*l],
                    _ => vec![],
                },
                match *r {
                    Oper::Var(_, _) | Oper::Temp(_, _) => vec![*r],
                    _ => vec![],
                },
            ]
            .concat(),
            Expr::Logical(l, _, r) => [
                match *l {
                    Oper::Var(_, _) | Oper::Temp(_, _) => vec![*l],
                    _ => vec![],
                },
                match *r {
                    Oper::Var(_, _) | Oper::Temp(_, _) => vec![*r],
                    _ => vec![],
                },
            ]
            .concat(),
            Expr::Phi(args) => {
                let mut used: Vec<Oper> = vec![];
                for arg in args {
                    match arg {
                        Oper::Var(_, _) | Oper::Temp(_, _) => used.push(*arg),
                        _ => (),
                    }
                }
                used
            }
            _ => vec![],
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
                    match *arg {
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
    Var(Identifier, SSA),
    Temp(Identifier, SSA),
    Constant(Constant),
}

impl Oper {
    pub fn as_non_ssa(&self) -> (usize, usize) {
        match self {
            Oper::Var(value, _) => (*value, 0),
            Oper::Temp(value, _) => (*value, 1),
            _ => panic!("Expected VAR or TEMP"),
        }
    }

    pub fn replace_with_ssa(&mut self, value: Oper, ssa: usize) {
        if let Oper::Var(v, s) = self {
            if Oper::Var(*v, *s) == value {
                *s = ssa;
            }
        }

        if let Oper::Temp(v, s) = self {
            if Oper::Temp(*v, *s) == value {
                *s = ssa;
            }
        }
    }
}

impl fmt::Debug for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Oper::Var(value, ssa) => write!(f, "{}.{}", get_string(*value), ssa),
            Oper::Temp(value, ssa) => write!(f, "_t{}.{}", value, ssa),
            Oper::Constant(c) => write!(f, "{}", c),
        }
    }
}
