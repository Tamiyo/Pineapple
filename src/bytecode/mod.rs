use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::get_string;
use std::fmt;

pub mod constant;
pub mod constant_pool;
pub mod distance;
pub mod string_intern;

type InternIndex = usize;
type Arity = usize;
type StackOffset = usize;
type RegisterIndex = usize;

#[derive(Copy, Clone)]
pub enum Label {
    Label(usize),
    Named(usize),
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Label::Label(l) => write!(f, "{}", l),
            Label::Named(l) => write!(f, "{}", get_string(*l)),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OR {
    REG(RegisterIndex),
    STACK(StackOffset),
}

#[derive(Debug, Copy, Clone)]
pub enum IR {
    REG(RegisterIndex),
    CONST(Constant),
    STACK(StackOffset),
    STACKPOP,
    RETVAL,
}

// TODO -> Abstract Label out to easily specify between named and unamed
//      -> in compiler fix up inreg
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    LABEL(Label),

    MOV(OR, IR),

    ADD(OR, IR, IR),
    SUB(OR, IR, IR),
    MUL(OR, IR, IR),
    DIV(OR, IR, IR),
    MOD(OR, IR, IR),
    POW(OR, IR, IR),
    // AND(OR, IR, IR),
    // OR(OR, IR, IR),
    LT(OR, IR, IR),
    LTE(OR, IR, IR),
    GT(OR, IR, IR),
    GTE(OR, IR, IR),
    EQ(OR, IR, IR),
    NEQ(OR, IR, IR),

    PUSH(IR),
    POP(OR),

    PUSHA,
    POPA,

    JUMP(Label),
    JUMPR(Label),

    NOP,

    BT(IR, Label), // Branch if true
    BF(IR, Label), // Banch if false

    CALL(InternIndex, Arity), // Function call
    RETURN(IR),

    HLT,
}
