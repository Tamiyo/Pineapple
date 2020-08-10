use crate::bytecode::constant::Constant;

pub mod constant;
pub mod distance;
pub mod string_intern;

/**
 *  [Instruction]
 *
 *  Instructions are the set of possible operations that
 *  the virtual machine can perform to run a program.
 */

//
pub type OR = usize;

type Label = usize;
type InternIndex = usize;
type Arity = usize;
type StackOffset = usize;

#[derive(Debug, Copy, Clone)]
pub enum IR {
    REG(usize),
    CONST(Constant),
    STACK(StackOffset),
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    LABEL(Label),

    /**
     *  [Load Constant]
     *  C = CONSTANT
     *
     *  Translates to:  LOAD C R[k]
     *  Semantically:   R[k] = C
     */
    // LOAD(OR),
    MOV(OR, IR),

    /**
     *  [Addition]
     *  
     *  Translates to:  ADD R[k] R[i] R[j]
     *  Semantically:   R[k] = R[i] + R[j]
     */
    ADD(OR, IR, IR),
    // SUB(OR, IR, IR),
    // MUL(OR, IR, IR),
    // DIV(OR, IR, IR),
    // MOD(OR, IR, IR),
    // POW(OR, IR, IR),
    // AND(OR, IR, IR),
    // OR(OR, IR, IR),
    LT(OR, IR, IR),
    LTE(OR, IR, IR),
    GT(OR, IR, IR),
    GTE(OR, IR, IR),

    EQ(OR, IR, IR),
    NEQ(OR, IR, IR),

    PUSH(IR),
    // POP(OR),

    // PUSHA
    // POPA
    JUMP(Label),

    // Branch if true
    BT(IR, Label),

    CALL(InternIndex, Arity),

    /**
     *  [Halt]
     *
     *  Halts the VM's execution at the current instruction.
     */
    HLT,
}
