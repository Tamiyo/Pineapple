use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::get_string;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;

use std::fmt;

pub mod translate;

type Label = usize;
type Interned = usize;
type Arity = usize;
type Value = usize;
type Offset = usize;
type IsVar = bool;

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Tac(Operand, Expr),
    Label(Label),
    NamedLabel(Label),
    Jump(Label),
    CJump(Expr, Label),
    Call(Interned, Arity),
    Return(Operand),
    StackPush(Operand),
    StackPop,
}

impl Stmt {
    pub fn has_label(&self, label: Label) -> bool {
        match self {
            Stmt::Label(l) => *l == label,
            Stmt::NamedLabel(nl) => *nl == label,
            Stmt::Jump(j) => *j == label,
            Stmt::CJump(_, cj) => *cj == label,
            _ => false,
        }
    }

    pub fn replace_label(&mut self, label: Label) {
        match self {
            Stmt::Label(l) => *l = label,
            Stmt::NamedLabel(nl) => *nl = label,
            Stmt::Jump(j) => *j = label,
            Stmt::CJump(_, cj) => *cj = label,
            _ => (),
        }
    }

    pub fn vars_defined(&self) -> Vec<Operand> {
        match self {
            Stmt::Tac(lval, rval) => lval.vars_defined(),
            _ => vec![],
        }
    }

    pub fn vars_used(&self) -> Vec<Operand> {
        match self {
            Stmt::Tac(lval, rval) => rval.vars_used(),
            Stmt::CJump(cond, label) => cond.vars_used(),
            Stmt::StackPush(target) => target.vars_used(),
            _ => vec![],
        }
    }

    pub fn uses_var(&self, v: Operand) -> bool {
        match self {
            Stmt::Tac(lval, rval) => rval.uses(v),
            Stmt::CJump(cond, label) => cond.uses(v),
            Stmt::StackPush(target) => *target == v,
            _ => false,
        }
    }

    pub fn replace_var_use_with_ssa(&mut self, value: usize, ssa: usize, is_var: bool) {
        match self {
            Stmt::Tac(lval, rval) => rval.replace_with_ssa(value, ssa, is_var),
            Stmt::CJump(cond, label) => cond.replace_with_ssa(value, ssa, is_var),
            Stmt::StackPush(target) => target.replace_with_ssa(value, ssa, is_var),
            _ => (),
        }
    }

    pub fn replace_var_def_with_ssa(&mut self, value: usize, ssa: usize, is_var: bool) {
        match self {
            Stmt::Tac(lval, rval) => lval.replace_with_ssa(value, ssa, is_var),
            _ => (),
        }
    }

    pub fn replace_all_operand_with(&mut self, v: Operand, c: Operand) {
        match self {
            Stmt::Tac(lval, rval) => {
                if *lval == v {
                    *lval = c;
                }
                rval.replace_operand_with(v, c);
            }
            Stmt::CJump(cond, _) => cond.replace_operand_with(v, c),
            Stmt::StackPush(target) => {
                if *target == v {
                    *target = c;
                }
            }
            _ => (),
        }
    }

    pub fn replace_operand_with(&mut self, v: Operand, c: Operand) {
        match self {
            Stmt::Tac(_, rval) => rval.replace_operand_with(v, c),
            Stmt::CJump(cond, _) => cond.replace_operand_with(v, c),
            Stmt::StackPush(target) => {
                if *target == v {
                    *target = c;
                }
            }
            _ => (),
        }
    }

    pub fn is_phi_constant(&self) -> Option<(Operand, Operand)> {
        match self {
            Stmt::Tac(lval, Expr::Phi(args)) => {
                if let Operand::Constant(c) = args[0] {
                    if args.iter().filter(|&n| *n == Operand::Constant(c)).count() == args.len() {
                        Some((*lval, args[0]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn patch_phi(&mut self, x: Operand, w: &mut Vec<Stmt>) {
        match self {
            Stmt::Tac(_, rval) => {
                if let Expr::Phi(args) = rval {
                    let mut i = 0;
                    while i < args.len() {
                        if args[i] == x {
                            args.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                    if args.len() == 1 {
                        *rval = Expr::Operand(args[0]);
                    }
                    w.push(self.clone());
                }
            }
            _ => (),
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Tac(lval, rval) => write!(f, "\t{:?} = {:?}", lval, rval),
            Stmt::Label(label) => write!(f, "_L{:?}:", label),
            Stmt::NamedLabel(label) => write!(f, "{:?}:", get_string(*label)),
            Stmt::Jump(label) => write!(f, "\tgoto _L{:?}", label),
            Stmt::CJump(cond, label) => write!(f, "\tif {:?} goto _L{:?}", cond, label),
            Stmt::Call(n, arity) => write!(f, "\tcall _{}({})", get_string(*n), arity),
            Stmt::Return(rval) => write!(f, "\tret {:?}", rval),
            Stmt::StackPush(rval) => write!(f, "\t_push {:?}", rval),
            Stmt::StackPop => write!(f, "\t_pop"),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Expr {
    Binary(Operand, BinOp, Operand),
    Logical(Operand, RelOp, Operand),
    Operand(Operand),
    Phi(Vec<Operand>),
    StackPop,
}

impl Expr {
    pub fn vars_used(&self) -> Vec<Operand> {
        match self {
            Expr::Operand(o) => o.vars_used(),
            Expr::Binary(l, _, r) => [&l.vars_used()[..], &r.vars_used()[..]].concat(),
            Expr::Logical(l, _, r) => [&l.vars_used()[..], &r.vars_used()[..]].concat(),
            Expr::Phi(args) => {
                let mut used: Vec<Operand> = vec![];
                for arg in args {
                    if let Operand::Assignable(_, _, _) = arg {
                        used.push(*arg)
                    }
                }
                used
            }
            _ => vec![],
        }
    }

    pub fn uses(&self, v: Operand) -> bool {
        match self {
            Expr::Operand(o) => *o == v,
            Expr::Binary(l, _, r) => *l == v || *r == v,
            Expr::Logical(l, _, r) => *l == v || *r == v,
            Expr::Phi(args) => {
                let mut used: Vec<Operand> = vec![];
                for arg in args {
                    if let Operand::Assignable(_, _, _) = arg {
                        used.push(*arg)
                    }
                }
                !used.is_empty()
            }
            _ => false,
        }
    }

    pub fn replace_with_ssa(&mut self, value: usize, ssa: usize, is_var: bool) {
        match self {
            Expr::Operand(o) => o.replace_with_ssa(value, ssa, is_var),
            Expr::Binary(l, _, r) => {
                l.replace_with_ssa(value, ssa, is_var);
                r.replace_with_ssa(value, ssa, is_var);
            }
            Expr::Logical(l, _, r) => {
                l.replace_with_ssa(value, ssa, is_var);
                r.replace_with_ssa(value, ssa, is_var);
            }
            _ => (),
        }
    }

    fn replace_operand_with(&mut self, v: Operand, c: Operand) {
        match self {
            Expr::Operand(o) => {
                if *o == v {
                    *o = c
                }
            }
            Expr::Binary(l, _, r) | Expr::Logical(l, _, r) => {
                if *l == v {
                    *l = c;
                }

                if *r == v {
                    *r = c;
                }
            }
            Expr::Phi(args) => {
                for arg in args {
                    if *arg == v {
                        *arg = c;
                    }
                }
            }
            _ => (),
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::Binary(l, o, r) => write!(f, "{:?} {:?} {:?}", l, o, r),
            Expr::Logical(l, o, r) => write!(f, "{:?} {:?} {:?}", l, o, r),
            Expr::Operand(o) => write!(f, "{:?}", o),
            Expr::Phi(args) => {
                write!(f, "Φ(")?;
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        Operand::Assignable(value, ssa, is_var) => {
                            if *is_var {
                                if i != args.len() - 1 {
                                    write!(f, "{}.{}, ", get_string(*value), ssa)?;
                                } else {
                                    write!(f, "{}.{}", get_string(*value), ssa)?;
                                }
                            } else if i != args.len() - 1 {
                                write!(f, "_t{}.{}, ", value, ssa)?;
                            } else {
                                write!(f, "_t{}.{}", value, ssa)?;
                            }
                        }
                        Operand::Constant(c) => {
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
            Expr::StackPop => write!(f, "_pop"),
            _ => unimplemented!(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Assignable(Value, Offset, IsVar),
    Constant(Constant),
    Register(usize), // Special Operand used during compilation to bytecode
    Return,
}

impl Operand {
    pub fn vars_defined(&self) -> Vec<Operand> {
        match self {
            Operand::Assignable(_, _, _) => vec![*self],
            _ => vec![],
        }
    }

    pub fn vars_used(&self) -> Vec<Operand> {
        match self {
            Operand::Assignable(_, _, _) => vec![*self],
            _ => vec![],
        }
    }

    pub fn replace_with_ssa(&mut self, value: usize, ssa: usize, is_var: bool) {
        if let Operand::Assignable(v, s, i) = self {
            if *v == value && *i == is_var {
                *s = ssa;
            }
        }
    }
}

impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Operand::Assignable(v, s, b) => {
                if *b {
                    write!(f, "{}.{}", get_string(*v), s)
                } else {
                    write!(f, "_t{}.{}", v, s)
                }
            }
            Operand::Constant(c) => write!(f, "{}", c),
            Operand::Register(r) => write!(f, "${}", r),
            Operand::Return => write!(f, "_r"),
        }
    }
}
