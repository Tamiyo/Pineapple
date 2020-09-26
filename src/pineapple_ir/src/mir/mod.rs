use crate::op::{BinOp, RelOp};
use crate::Value;
use crate::ValueTy;

type BlockIndex = usize;
type Interned = usize;
type Arity = usize;
type Sym = usize;
type Version = usize;

type StatementIndex = usize;

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
    //  See SSA-Book p. 36z
    ParallelCopy(Vec<StatementIndex>),
}

impl Stmt {
    pub fn oper_defined(&self) -> Vec<Oper> {
        match self {
            Stmt::Tac(lval, _) => match lval {
                Oper::SSA(_) => vec![*lval],
                _ => vec![],
            },
            _ => vec![],
        }
    }

    pub fn oper_used(&self) -> Vec<Oper> {
        match self {
            Stmt::Tac(_, rval) => rval.oper_used(),
            Stmt::CJump(cond, _) => cond.oper_used(),
            Stmt::StackPush(oper) => match oper {
                Oper::SSA(_) => vec![*oper],
                _ => vec![],
            },
            Stmt::CastAs(oper, _) => vec![*oper],
            Stmt::Return(oper) => match *oper {
                Some(Oper::SSA(_)) => vec![oper.unwrap()],
                _ => vec![],
            },
            _ => vec![],
        }
    }

    pub fn replace_all_oper_def_with(&mut self, a: &Oper, b: &Oper) {
        match self {
            Stmt::Tac(lval, _) => lval.replace_oper_with(a, b),
            _ => (),
        }
    }

    pub fn replace_all_oper_use_with(&mut self, a: &Oper, b: &Oper) {
        match self {
            Stmt::Tac(_, rval) => rval.replace_oper_with(a, b),
            Stmt::CJump(cond, _) => cond.replace_oper_with(a, b),
            Stmt::StackPush(oper) => oper.replace_oper_with(a, b),
            Stmt::CastAs(oper, _) => oper.replace_oper_with(a, b),
            Stmt::Return(oper) => {
                if let Some(oper) = oper {
                    oper.replace_oper_with(a, b)
                }
            }
            _ => (),
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
            Stmt::StackPush(oper) => oper.replace_with_ssa(value, ssa),
            Stmt::CastAs(oper, _) => oper.replace_with_ssa(value, ssa),
            Stmt::Return(oper) => {
                if let Some(oper) = oper {
                    oper.replace_with_ssa(value, ssa)
                }
            }
            _ => (),
        }
    }
}

impl std::fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Tac(lval, rval) => write!(f, "{:?} = {:?}", lval, rval),
            Stmt::Label(label) => match label {
                Label::Marker(label) => write!(f, "_L{}:", label),
                Label::Named(label) => write!(f, "_{}:", label),
            },
            Stmt::CastAs(oper, t) => write!(f, "cast {:?} as {:?}", oper, t),
            Stmt::Jump(label) => write!(f, "goto _L{:?}", label),
            Stmt::CJump(cond, label) => write!(f, "if {:?} goto _L{:?}", cond, label),
            Stmt::ParallelCopy(copies) => {
                for copy in copies {
                    write!(f, "(p) {:?}", copy)?;
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

impl Expr {
    pub fn oper_used(&self) -> Vec<Oper> {
        match self {
            Expr::Oper(o) => match *o {
                Oper::SSA(_) => vec![*o],
                _ => vec![],
            },
            Expr::Binary(l, _, r) => [
                match *l {
                    Oper::SSA(_) => vec![*l],
                    _ => vec![],
                },
                match *r {
                    Oper::SSA(_) => vec![*r],
                    _ => vec![],
                },
            ]
            .concat(),
            Expr::Logical(l, _, r) => [
                match *l {
                    Oper::SSA(_) => vec![*l],
                    _ => vec![],
                },
                match *r {
                    Oper::SSA(_) => vec![*r],
                    _ => vec![],
                },
            ]
            .concat(),
            Expr::Phi(args) => {
                let mut used: Vec<Oper> = vec![];
                for arg in args {
                    if let Oper::SSA(SSA::Var(_, _)) = arg.0 {
                        used.push(arg.0);
                    }
                }
                used
            }
            _ => vec![],
        }
    }

    pub fn replace_oper_with(&mut self, a: &Oper, b: &Oper) {
        match self {
            Expr::Oper(o) => o.replace_oper_with(a, b),
            Expr::Binary(l, _, r) => {
                l.replace_oper_with(a, b);
                r.replace_oper_with(a, b);
            }
            Expr::Logical(l, _, r) => {
                l.replace_oper_with(a, b);
                r.replace_oper_with(a, b);
            }
            Expr::Phi(args) => {
                for arg in args {
                    if arg.0 == *a {
                        arg.0 = *b;
                    }
                }
            }
            _ => (),
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
                        Oper::SSA(ssa) => {
                            if i != args.len() - 1 {
                                write!(f, "{:?}, ", ssa)?;
                            } else {
                                write!(f, "{:?}", ssa)?;
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SSA {
    Var(Sym, Version),
    Temp(Sym, Version),
}

impl std::fmt::Debug for SSA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SSA::Var(value, ssa) => write!(f, "_v{}.{}", value, ssa),
            SSA::Temp(value, ssa) => write!(f, "_t{}.{}", value, ssa),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Oper {
    SSA(SSA),
    Value(Value),

    StackPop,

    Register(usize),
    ReturnValue,
    StackLocation(usize),
}

impl Oper {
    pub fn replace_oper_with(&mut self, a: &Oper, b: &Oper) {
        if self == a {
            *self = *b;
        } else {
        }
    }

    pub fn replace_with_ssa(&mut self, value: Oper, ssa: usize) {
        if let Oper::SSA(SSA::Var(v, s)) = self {
            if let Oper::SSA(SSA::Var(a, _)) = value {
                if *v == a {
                    *s = ssa;
                }
            }
        } else if let Oper::SSA(SSA::Temp(v, s)) = self {
            if let Oper::SSA(SSA::Temp(a, _)) = value {
                if *v == a {
                    *s = ssa;
                }
            }
        }
    }
}

impl std::fmt::Debug for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Oper::Register(reg) => write!(f, "${}", reg),
            Oper::StackLocation(offset) => write!(f, "sp[-{}]", offset),
            Oper::StackPop => write!(f, "_pop"),
            Oper::ReturnValue => write!(f, "$rv"),
            Oper::SSA(ssa) => write!(f, "{:?}", ssa),
            Oper::Value(c) => write!(f, "{:?}", c),
        }
    }
}
