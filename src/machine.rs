use crate::term::Term;

/// A minimal set of instructions for our abstract machine.
/// We start with only two instructions:
/// - `PutConst`: Puts a constant in a register.
/// - `PutVar`: Puts a variable in a register.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    PutConst { register: usize, value: i32 },
    PutVar { register: usize, name: String },
}

/// The abstract machine structure.
#[derive(Debug)]
pub struct Machine {
    /// Registers: each can hold an optional Term.
    pub registers: Vec<Option<Term>>,
    /// The code (instructions) for the machine.
    pub code: Vec<Instruction>,
    /// Program counter.
    pub pc: usize,
}

impl Machine {
    /// Create a new machine with a specified number of registers and given code.
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
        }
    }

    /// Execute one instruction.
    ///
    /// Returns `false` if there are no more instructions.
    pub fn step(&mut self) -> bool {
        if self.pc >= self.code.len() {
            return false;
        }
        let instr = self.code[self.pc].clone();
        self.pc += 1;
        match instr {
            Instruction::PutConst { register, value } => {
                if register < self.registers.len() {
                    self.registers[register] = Some(Term::Const(value));
                } else {
                    eprintln!("Error: Register {} out of bounds", register);
                }
            }
            Instruction::PutVar { register, name } => {
                if register < self.registers.len() {
                    self.registers[register] = Some(Term::Var(name));
                } else {
                    eprintln!("Error: Register {} out of bounds", register);
                }
            }
        }
        true
    }

    /// Run the machine until no more instructions are available.
    pub fn run(&mut self) {
        while self.step() {}
    }
}
