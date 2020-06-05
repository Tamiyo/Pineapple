use crate::bytecode::instruction::Instruction;

#[derive(Debug)]
pub struct VM {
    /**
     *  [Instruction Pointer]
     *
     *  Points to the current instruction being executed.
     */
    ip: usize,

    /**
     *  [Frame Pointer]
     *
     *  Points to the current frame, at the top of the function call stack.
     */
    fp: usize,

    /**
     *  [Registers]
     *  
     *  p = parameters
     *  t = locals
     *
     *  R[0]               ->  return values
     *  R[1] ... R[p]      ->  parameter registers
     *  R[p+1] ... R[p+t]  ->  local registers
     */
    r: [i32; 16],
}

impl VM {
    /**
     *  [Create a new VM]
     *
     *  Creates a new virtual machine with default parameters.
     */
    pub fn new() -> Self {
        VM {
            ip: 0,
            fp: 0,
            r: [0; 16],
        }
    }

    /**
     *  Dispatches the current instruction, executing the specified
     *  virtual machine operations that align with each instruction.
     *
     *  Returns:
     *      Ok(()) : Execution had no problems.
     *      Err(e) : An error occured at some stage of execution.
     */
    pub fn dispatch(&mut self, _instruction: Instruction) -> Result<(), String> {
        // match instruction {
        //     Instruction::LOAD(k, c) => {
        //         self.R[k] = c;
        //     }
        //     Instruction::ADD(k, i, j) => {
        //         self.R[k] = self.R[i] + self.R[j];
        //     }
        //     _ => ()
        // }

        Ok(())
    }
}
