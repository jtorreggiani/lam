// src/machine.rs

use crate::term::Term;
use std::collections::HashMap;

// The set of instructions for our abstract machine.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    // Puts a constant in a register.
    PutConst { register: usize, value: i32 },
    // Puts a variable in a register.
    PutVar { register: usize, name: String },
    // Unifies the term in the register with the given constant.
    GetConst { register: usize, value: i32 },
    // Unifies the term in the register with a variable.
    GetVar { register: usize, name: String },
    // Calls a predicate by name.
    Call { predicate: String },
    // Proceeds (returns) from the current predicate.
    Proceed,
    // Creates a choice point (for backtracking).
    Choice,
    // Fails and triggers backtracking.
    Fail,
    // The constructed term (Compound(functor, args)) is stored in `target`.
    BuildCompound { target: usize, functor: String, arg_registers: Vec<usize> },
    // Checks that the term in the specified register is a compound term with the given functor and arity.
    GetStructure { register: usize, functor: String, arity: usize },
}

// A frame to store return information for a predicate call.
#[derive(Debug, Clone)]
pub struct Frame {
    pub return_pc: usize,
}

// A choice point to support backtracking.
// Saves the program counter, registers, and substitution environment.
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: HashMap<String, Term>,
    pub saved_trail_len: usize,
}

#[derive(Debug, Clone)]
pub struct TrailEntry {
    pub variable: String,
    pub previous_value: Option<Term>,
}

// The abstract machine structure.
#[derive(Debug)]
pub struct Machine {
    // Registers: each can hold an optional Term.
    pub registers: Vec<Option<Term>>,
    // The code (instructions) for the machine.
    pub code: Vec<Instruction>,
    // Program counter.
    pub pc: usize,
    // Substitution environment mapping variable names to Terms.
    pub substitution: HashMap<String, Term>,
    // A control stack to hold frames for predicate calls.
    pub control_stack: Vec<Frame>,
    // A predicate table mapping predicate names to code addresses.
    pub predicate_table: HashMap<String, usize>,
    // A stack to hold choice points for backtracking.
    pub choice_stack: Vec<ChoicePoint>,
    // A trail to record variable bindings (for backtracking).
    pub trail: Vec<TrailEntry>,
}

impl Machine {
    // Create a new machine with a specified number of registers and given code.
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
            substitution: HashMap::new(),
            control_stack: Vec::new(),
            predicate_table: HashMap::new(),
            choice_stack: Vec::new(),
            trail: Vec::new(), 
        }
    }

    // Register a predicate with its starting code address.
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table.insert(name, address);
    }

    // Resolve a term to its current bound value, if any.
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

    // Unify two terms.
    ///
    // If one term is an unbound variable, bind it to the other.
    // If both are constants, they must be equal.
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
            (Term::Compound(functor1, args1), Term::Compound(functor2, args2)) => {
                if functor1 == functor2 && args1.len() == args2.len() {
                    for (a, b) in args1.iter().zip(args2.iter()) {
                        if !self.unify(a, b) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    // Execute one instruction.
    ///
    // Returns `false` if there are no more instructions or a failure terminates execution.
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
          },
          Instruction::PutVar { register, name } => {
              if register < self.registers.len() {
                  self.registers[register] = Some(Term::Var(name));
              } else {
                  eprintln!("Error: Register {} out of bounds", register);
              }
          },
          Instruction::GetConst { register, value } => {
              if register < self.registers.len() {
                  if let Some(term) = self.registers[register].clone() {
                      let goal = Term::Const(value);
                      if !self.unify(&term, &goal) {
                          eprintln!("Unification failed: cannot unify {:?} with {:?}", term, goal);
                      }
                  } else {
                      eprintln!("Error: Register {} is uninitialized", register);
                  }
              } else {
                  eprintln!("Error: Register {} out of bounds", register);
              }
          },
          Instruction::GetVar { register, name } => {
              if register < self.registers.len() {
                  if let Some(term) = self.registers[register].clone() {
                      let goal = Term::Var(name);
                      if !self.unify(&goal, &term) {
                          eprintln!("Unification failed: cannot unify {:?} with {:?}", goal, term);
                      }
                  } else {
                      eprintln!("Error: Register {} is uninitialized", register);
                  }
              } else {
                  eprintln!("Error: Register {} out of bounds", register);
              }
          },
          Instruction::Call { predicate } => {
              if let Some(&jump_to) = self.predicate_table.get(&predicate) {
                  self.control_stack.push(Frame { return_pc: self.pc });
                  println!("Calling predicate: {}", predicate);
                  self.pc = jump_to;
              } else {
                  eprintln!("Call failed: predicate {} not found", predicate);
              }
          },
          Instruction::Proceed => {
              if let Some(frame) = self.control_stack.pop() {
                  self.pc = frame.return_pc;
                  println!("Proceed: returning to pc = {}", self.pc);
              } else {
                  println!("Proceed: no frame to return to, finishing execution.");
              }
          },
          Instruction::Choice => {
              let cp = ChoicePoint {
                  saved_pc: self.pc,
                  saved_registers: self.registers.clone(),
                  saved_substitution: self.substitution.clone(),
                  saved_trail_len: self.trail.len(),
              };
              self.choice_stack.push(cp);
              println!("Choice point created.");
          },
          Instruction::Fail => {
              if let Some(cp) = self.choice_stack.pop() {
                  // Unwind the trail to restore variable bindings.
                  while self.trail.len() > cp.saved_trail_len {
                      if let Some(entry) = self.trail.pop() {
                          match entry.previous_value {
                              Some(prev_val) => {
                                  self.substitution.insert(entry.variable, prev_val);
                              }
                              None => {
                                  self.substitution.remove(&entry.variable);
                              }
                          }
                      }
                  }
                  // Restore registers and substitution, then set pc.
                  self.registers = cp.saved_registers;
                  self.substitution = cp.saved_substitution;
                  self.pc = cp.saved_pc + 2; // Skip the failed alternative.
                  println!("Backtracked to choice point at pc = {}", self.pc);
              } else {
                  println!("Fail: no choice point available. Terminating.");
                  return false;
              }
          },
          Instruction::GetStructure { register, functor, arity } => {
              if register < self.registers.len() {
                  if let Some(term) = self.registers[register].clone() {
                      match term {
                          Term::Compound(ref f, ref args) => {
                              if f == &functor && args.len() == arity {
                                  // Success: the structure matches.
                                  // (In a more advanced implementation, you might store the arguments
                                  // in designated registers for further unification.)
                                  println!("GetStructure succeeded: found {} with arity {}", functor, arity);
                              } else {
                                  eprintln!(
                                      "GetStructure failed: expected {} with arity {}, found {} with arity {}",
                                      functor, arity, f, args.len()
                                  );
                              }
                          },
                          _ => {
                              eprintln!("GetStructure failed: expected a compound term in register {}", register);
                          },
                      }
                  } else {
                      eprintln!("Error: Register {} is uninitialized", register);
                  }
              } else {
                  eprintln!("Error: Register {} out of bounds", register);
              }
          },
          Instruction::BuildCompound { target, functor, arg_registers } => {
              // Gather arguments from the specified registers.
              let mut args = Vec::new();
              for &reg in arg_registers.iter() {
                  if reg < self.registers.len() {
                      if let Some(term) = self.registers[reg].clone() {
                          args.push(term);
                      } else {
                          eprintln!("Error: Register {} is uninitialized", reg);
                          return true; // or handle error appropriately
                      }
                  } else {
                      eprintln!("Error: Register {} out of bounds", reg);
                      return true; // or handle error appropriately
                  }
              }
              // Construct the compound term and store it in the target register.
              if target < self.registers.len() {
                  self.registers[target] = Some(Term::Compound(functor, args));
              } else {
                  eprintln!("Error: Target register {} out of bounds", target);
              }
          },
      }
      true
  }

    // Run the machine until no more instructions are available.
    pub fn run(&mut self) {
        while self.step() {}
    }
}
