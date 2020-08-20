use crate::bytecode::constant::Constant;

pub mod constant;
pub mod distance;
pub mod string_intern;
pub mod constant_pool;

type Label = usize;
type InternIndex = usize;
type Arity = usize;
type StackOffset = usize;
type RegisterIndex = usize;

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
}

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    LABEL(Label),

    MOV(OR, IR),

    ADD(OR, IR, IR),
    // SUB(OR, IR, IR),
    // MUL(OR, IR, IR),
    // DIV(OR, IR, IR),
    // MOD(OR, IR, IR),
    // POW(OR, IR, IR),
    // AND(OR, IR, IR),
    // OR(OR, IR, IR),
    // LT(OR, IR, IR),
    // LTE(OR, IR, IR),
    // GT(OR, IR, IR),
    // GTE(OR, IR, IR),
    EQ(OR, IR, IR),
    NEQ(OR, IR, IR),

    PUSH(IR),
    // POP(OR),

    // PUSHA
    // POPA
    JUMP(Label),

    BT(IR, Label),    // Branch if true
    BF(IR, Label), // Banch if false

    CALL(InternIndex, Arity), // Function call

    HLT,
}
