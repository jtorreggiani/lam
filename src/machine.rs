// src/machine.rs

use crate::term::Term;
use std::collections::HashMap;

/// A minimal set of instructions for our abstract machine.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    /// Puts a constant in a register.
    PutConst { register: usize, value: i32 },
    /// Puts a variable in a register.
    PutVar { register: usize, name: String },
    /// Unifies the term in the register with the given constant.
    GetConst { register: usize, value: i32 },
    /// Unifies the term in the register with a variable.
    GetVar { register: usize, name: String },
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
    /// Substitution environment mapping variable names to Terms.
    pub substitution: HashMap<String, Term>,
}

impl Machine {
    /// Create a new machine with a specified number of registers and given code.
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
            substitution: HashMap::new(),
        }
    }

    /// Resolve a term to its current bound value, if any.
    fn resolve(&self, term: &Term) -> Term {
        match term {
            Term::Var(name) => {
                if let Some(bound) = self.substitution.get(name) {
                    // Recursively resolve in case of chained bindings.
                    self.resolve(bound)
                } else {
                    term.clone()
                }
            }
            _ => term.clone(),
        }
    }

    /// Unify two terms.
    ///
    /// If one term is a variable that is not bound, bind it to the other.
    /// If both are constants, they must be equal.
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> bool {
        let term1 = self.resolve(t1);
        let term2 = self.resolve(t2);
        match (term1, term2) {
            (Term::Const(a), Term::Const(b)) => a == b,
            (Term::Var(name), other) => {
                self.substitution.insert(name, other);
                true
            }
            (other, Term::Var(name)) => {
                self.substitution.insert(name, other);
                true
            }
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
            Instruction::GetConst { register, value } => {
                if register < self.registers.len() {
                    if let Some(term) = self.registers[register].clone() {
                        let goal = Term::Const(value);
                        if !self.unify(&term, &goal) {
                            eprintln!(
                                "Unification failed: cannot unify {:?} with {:?}",
                                term, goal
                            );
                        }
                    } else {
                        eprintln!("Error: Register {} is uninitialized", register);
                    }
                } else {
                    eprintln!("Error: Register {} out of bounds", register);
                }
            }
            Instruction::GetVar { register, name } => {
                if register < self.registers.len() {
                    if let Some(term) = self.registers[register].clone() {
                        let goal = Term::Var(name);
                        // **Note:** Reverse the order of arguments so that the new variable (goal)
                        // is bound to the stored value.
                        if !self.unify(&goal, &term) {
                            eprintln!(
                                "Unification failed: cannot unify {:?} with {:?}",
                                goal, term
                            );
                        }
                    } else {
                        eprintln!("Error: Register {} is uninitialized", register);
                    }
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
