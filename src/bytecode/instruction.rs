/**
 *  [Instruction]
 *
 *  Instructions are the set of possible operations that
 *  the virtual machine can perform to run a program.
 */

type OR = usize;
type IR = usize;

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /**
     *  [Load Constant]
     *  C = CONSTANT
     *
     *  Translates to:  LOAD C R[k]
     *  Semantically:   R[k] = C
     */
    LOAD,

    /**
     *  [Addition]
     *  
     *  Translates to:  ADD R[k] R[i] R[j]
     *  Semantically:   R[k] = R[i] + R[j]
     */
    ADD(OR, IR, IR),

    /**
     *  [Halt]
     *
     *  Halts the VM's execution at the current instruction.
     */
    HLT,

    /**
     *  [Illegal Instruction]
     *
     *  Called when an invalid instruction is given.
     */
    IGL,
}
