use crate::bytecode::constant::Constant;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;
use std::ops::Deref;

use crate::bytecode::string_intern::get_string;

use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Label {
    Named(usize),
    Label(usize),
}

impl Deref for Label {
    type Target = usize;

    fn deref(&self) -> &usize {
        match self {
            Label::Named(n) | Label::Label(n) => &n,
        }
    }
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Label::Named(n) => write!(f, "begin {}", get_string(*n)),
            Label::Label(n) => write!(f, "_L{}:", n),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SSA {
    pub value: usize,
    pub ssa: usize,
    pub is_var: bool,
}

impl fmt::Debug for SSA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if !self.is_var {
            write!(f, "_t{}.{}", self.value, self.ssa)
        } else {
            write!(f, "{}.{}", get_string(self.value), self.ssa)
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Operand {
    Assignable(SSA),
    Constant(Constant),
}

impl Operand {
    pub fn used(&self) -> Vec<SSA> {
        match self {
            Operand::Assignable(v) => vec![*v],
            _ => vec![],
        }
    }

    pub fn uses(&self, v: Operand) -> bool {
        *self == v
    }

    pub fn replace_with_ssa(&mut self, value: usize, is_var: bool, ssa: usize) {
        if let Operand::Assignable(v) = self {
            if v.value == value && v.is_var == is_var {
                v.ssa = ssa
            }
        }
    }

    pub fn replace_with(&mut self, v: Operand, c: Operand) {
        if *self == v {
            *self = c;
        }
    }
}

impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Operand::Assignable(v) => {
                if v.is_var {
                    write!(f, "{}.{}", get_string(v.value), v.ssa)
                } else {
                    write!(f, "_t{}.{}", v.value, v.ssa)
                }
            }
            Operand::Constant(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Jump {
    pub goto: Label,
}

#[derive(Clone, PartialEq)]
pub struct CJump {
    pub cond: Expr,
    pub goto: Label,
}

impl CJump {
    pub fn used(&self) -> Vec<SSA> {
        self.cond.used()
    }

    pub fn uses(&self, v: Operand) -> bool {
        self.cond.uses(v)
    }

    pub fn is_constant_jump(&self) -> Option<Constant> {
        self.cond.is_constant_logical()
    }

    pub fn replace_use_with_ssa(&mut self, value: usize, is_value: bool, ssa: usize) {
        self.cond.replace_with_ssa(value, is_value, ssa);
    }

    fn replace_with(&mut self, v: Operand, c: Operand) {
        self.cond.replace_with(v, c);
    }
}

#[derive(Clone, PartialEq)]
pub enum Expr {
    Binary(Operand, BinOp, Operand),
    Logical(Operand, RelOp, Operand),
    Operand(Operand),
    Phi(Vec<Operand>), // Special for SSA
    Call(usize),
    StackPop,
}

impl Expr {
    pub fn used(&self) -> Vec<SSA> {
        match self {
            Expr::Operand(o) => o.used(),
            Expr::Binary(l, _, r) => [&l.used()[..], &r.used()[..]].concat(),
            Expr::Logical(l, _, r) => [&l.used()[..], &r.used()[..]].concat(),
            Expr::Phi(args) => {
                let mut used: Vec<SSA> = vec![];
                for arg in args {
                    if let Operand::Assignable(v) = arg {
                        used.push(*v)
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
                let mut used: Vec<SSA> = vec![];
                for arg in args {
                    if let Operand::Assignable(v) = arg {
                        used.push(*v)
                    }
                }
                !used.is_empty()
            }
            _ => false,
        }
    }

    fn is_constant_logical(&self) -> Option<Constant> {
        if let Expr::Operand(Operand::Constant(c)) = &self {
            Some(*c)
        } else {
            None
        }
    }

    pub fn replace_with_ssa(&mut self, value: usize, is_value: bool, ssa: usize) {
        match self {
            Expr::Operand(o) => o.replace_with_ssa(value, is_value, ssa),
            Expr::Binary(l, _, r) => {
                l.replace_with_ssa(value, is_value, ssa);
                r.replace_with_ssa(value, is_value, ssa);
            }
            Expr::Logical(l, _, r) => {
                l.replace_with_ssa(value, is_value, ssa);
                r.replace_with_ssa(value, is_value, ssa);
            }
            _ => (),
        }
    }

    pub fn replace_phi_def(&mut self, j: usize, ssa: usize) {
        if let Expr::Phi(args) = self {
            let operand = &args[j];
            if let Operand::Assignable(v) = operand {
                args[j] = Operand::Assignable(SSA {
                    value: v.value,
                    ssa: ssa,
                    is_var: v.is_var,
                });
            }
        }
    }

    fn replace_with(&mut self, v: Operand, c: Operand) {
        match self {
            Expr::Operand(o) => *o = c,
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
                        Operand::Assignable(v) => {
                            if v.is_var {
                                if i != args.len() - 1 {
                                    write!(f, "{}.{}, ", get_string(v.value), v.ssa)?;
                                } else {
                                    write!(f, "{}.{}", get_string(v.value), v.ssa)?;
                                }
                            } else if i != args.len() - 1 {
                                write!(f, "_t{}.{}, ", v.value, v.ssa)?;
                            } else {
                                write!(f, "_t{}.{}", v.value, v.ssa)?;
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
            Expr::Call(fname) => write!(f, "Call {}", get_string(*fname)),
            Expr::StackPop => write!(f, "_pop"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Tac {
    pub lval: Operand,
    pub rval: Expr,
}

impl Tac {
    pub fn used(&self) -> Vec<SSA> {
        self.rval.used()
    }

    pub fn uses(&self, v: Operand) -> bool {
        self.rval.uses(v)
    }

    pub fn vars_defined(&self) -> Vec<SSA> {
        self.lval.used()
    }

    pub fn replace_use_with_ssa(&mut self, value: usize, is_value: bool, ssa: usize) {
        self.rval.replace_with_ssa(value, is_value, ssa)
    }

    pub fn replace_def_with_ssa(&mut self, value: usize, is_value: bool, ssa: usize) {
        self.lval.replace_with_ssa(value, is_value, ssa)
    }

    fn is_phi_constant(&self) -> Option<(Operand, Operand)> {
        match &self.rval {
            Expr::Phi(args) => {
                if let Operand::Constant(c) = args[0] {
                    if args.iter().filter(|&n| *n == Operand::Constant(c)).count() == args.len() {
                        Some((self.lval, args[0]))
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

    fn is_constant(&self) -> Option<(Operand, Operand)> {
        if let Expr::Operand(Operand::Constant(c)) = &self.rval {
            Some((self.lval, Operand::Constant(*c)))
        } else {
            None
        }
    }

    fn is_single_assignment(&self) -> Option<(Operand, Operand)> {
        if let Expr::Operand(o) = self.rval {
            Some((self.lval, o))
        } else {
            None
        }
    }

    fn is_constant_binary(&self) -> Option<(Constant, BinOp, Constant)> {
        if let Expr::Binary(Operand::Constant(c1), op, Operand::Constant(c2)) = &self.rval {
            Some((*c1, *op, *c2))
        } else {
            None
        }
    }

    fn is_constant_logical(&self) -> Option<(Constant, RelOp, Constant)> {
        if let Expr::Logical(Operand::Constant(c1), op, Operand::Constant(c2)) = &self.rval {
            Some((*c1, *op, *c2))
        } else {
            None
        }
    }

    fn is_function_pop(&self) -> bool {
        match &self.rval {
            Expr::StackPop => true,
            _ => false,
        }
    }

    fn replace_with(&mut self, v: Operand, c: Operand) {
        self.rval.replace_with(v, c)
    }

    fn replace_rval_with(&mut self, y: Operand) {
        self.rval = Expr::Operand(y)
    }

    fn patch_phi(&mut self, x: SSA, w: &mut Vec<Stmt>) {
        if let Expr::Phi(args) = &mut self.rval {
            let mut i = 0;
            while i < args.len() {
                if args[i] == Operand::Assignable(x) {
                    args.remove(i);
                } else {
                    i += 1;
                }
            }

            if args.len() == 1 {
                self.rval = Expr::Operand(args[0]);
            }

            w.push(Stmt::Tac(self.clone()));
        }
    }
}

impl fmt::Debug for Tac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?} = {:?}", self.lval, self.rval)
    }
}

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Tac(Tac),
    Label(Label),
    Jump(Jump),
    CJump(CJump),
    Call(usize),
    StackPush(Operand), // pushes an operand onto the stack
    StackPop,           // pops a elements from the stack
}

impl Stmt {
    pub fn has_label(&self, label: Label) -> bool {
        match self {
            Stmt::Label(l) => *l == label,
            Stmt::Jump(j) => j.goto == label,
            Stmt::CJump(cj) => cj.goto == label,
            _ => false,
        }
    }

    pub fn change_label(&mut self, label: Label) {
        match self {
            Stmt::Label(l) => *l = label,
            Stmt::Jump(j) => j.goto = label,
            Stmt::CJump(cj) => cj.goto = label,
            _ => (),
        }
    }

    pub fn used(&self) -> Vec<SSA> {
        match self {
            Stmt::Tac(tac) => tac.used(),
            Stmt::CJump(cj) => cj.used(),
            Stmt::StackPush(o) => o.used(),
            _ => vec![],
        }
    }

    pub fn uses(&self, v: Operand) -> bool {
        match self {
            Stmt::Tac(tac) => tac.uses(v),
            Stmt::CJump(cj) => cj.uses(v),
            Stmt::StackPush(o) => o.uses(v),
            _ => false,
        }
    }

    pub fn defined(&self) -> Vec<SSA> {
        match self {
            Stmt::Tac(tac) => tac.vars_defined(),
            _ => vec![],
        }
    }

    pub fn replace_use_with_ssa(&mut self, value: usize, is_value: bool, ssa: usize) {
        match self {
            Stmt::Tac(tac) => tac.replace_use_with_ssa(value, is_value, ssa),
            Stmt::CJump(cj) => cj.replace_use_with_ssa(value, is_value, ssa),
            Stmt::StackPush(o) => o.replace_with_ssa(value, is_value, ssa),
            _ => (),
        }
    }

    pub fn replace_def_with_ssa(&mut self, value: usize, is_value: bool, ssa: usize) {
        if let Stmt::Tac(tac) = self {
            tac.replace_def_with_ssa(value, is_value, ssa)
        }
    }

    // S is v <- Φ(c, c, ..., c) for some constant c
    pub fn is_phi_constant(&self) -> Option<(Operand, Operand)> {
        match self {
            Stmt::Tac(tac) => tac.is_phi_constant(),
            _ => None,
        }
    }

    // S is v <- c for some constant c
    pub fn is_constant(&self) -> Option<(Operand, Operand)> {
        match self {
            Stmt::Tac(tac) => tac.is_constant(),
            _ => None,
        }
    }

    pub fn is_single_assignment(&self) -> Option<(Operand, Operand)> {
        match self {
            Stmt::Tac(tac) => tac.is_single_assignment(),
            _ => None,
        }
    }

    pub fn is_constant_binary(&self) -> Option<(Constant, BinOp, Constant)> {
        match self {
            Stmt::Tac(tac) => tac.is_constant_binary(),
            _ => None,
        }
    }

    pub fn is_constant_logical(&self) -> Option<(Constant, RelOp, Constant)> {
        match self {
            Stmt::Tac(tac) => tac.is_constant_logical(),
            _ => None,
        }
    }

    pub fn is_function_pop(&self) -> bool {
        match self {
            Stmt::Tac(tac) => tac.is_function_pop(),
            _ => false,
        }
    }

    pub fn is_constant_jump(&self) -> Option<Constant> {
        match self {
            Stmt::CJump(cj) => cj.is_constant_jump(),
            _ => None,
        }
    }

    pub fn replace_with(&mut self, v: Operand, c: Operand) {
        match self {
            Stmt::Tac(tac) => tac.replace_with(v, c),
            Stmt::CJump(cj) => cj.replace_with(v, c),
            Stmt::StackPush(o) => o.replace_with(v, c),
            _ => (),
        }
    }

    pub fn replace_rval_with(&mut self, y: Operand) {
        if let Stmt::Tac(tac) = self {
            tac.replace_rval_with(y)
        }
    }

    pub fn patch_phi(&mut self, x: SSA, w: &mut Vec<Stmt>) {
        if let Stmt::Tac(tac) = self {
            tac.patch_phi(x, w)
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Tac(t) => write!(f, "\t{:?}", t),
            Stmt::Label(l) => write!(f, "{:?}", l),
            Stmt::Jump(j) => write!(f, "\tgoto {:?}", j.goto),
            Stmt::CJump(cj) => write!(f, "\tif {:?} goto {:?}", cj.cond, cj.goto),
            Stmt::Call(n) => write!(f, "\tcall _{}", get_string(*n)),
            Stmt::StackPush(o) => write!(f, "\t_push {:?}", o),
            Stmt::StackPop => write!(f, "\t_pop"),
        }
    }
}
