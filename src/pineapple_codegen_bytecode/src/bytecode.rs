use pineapple_ir::mir::Label;
use pineapple_ir::ValueTy;

type InternIndex = usize;
type Arity = usize;
type MemoryLocation = usize;
type StackOffset = usize;
type RegisterIndex = usize;
type ValueIndex = usize;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OR {
    REG(RegisterIndex),
    STACK(StackOffset),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    REG(RegisterIndex),
    VALUE(ValueIndex),
    STACK(StackOffset),
    MEMLOC(MemoryLocation),
    STACKPOP,
    RETVAL,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    LABEL(Label),

    MOV(OR, IR),

    CAST(OR, ValueTy),

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

    NOP,

    BT(IR, Label), // Branch if true
    BF(IR, Label), // Banch if false

    CALL(InternIndex, Arity), // Function call
    RETURN(IR),

    HLT,
}
