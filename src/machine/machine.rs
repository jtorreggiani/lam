use std::collections::HashMap;
use crate::term::Term;
use crate::arithmetic;

use super::{
    instruction::Instruction,
    frame::Frame,
    choice_point::ChoicePoint,
    trail::TrailEntry,
};

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
    // A predicate table mapping predicate names to lists of clause addresses.
    pub predicate_table: HashMap<String, Vec<usize>>,
    // A stack to hold choice points for backtracking.
    pub choice_stack: Vec<ChoicePoint>,
    // A trail to record variable bindings (for backtracking).
    pub trail: Vec<TrailEntry>,
    // A stack of environment frames for local variables.
    pub environment_stack: Vec<Vec<Option<Term>>>,
    // Index table mapping predicate names to an index mapping.
    // For each predicate, the key is a Term (the index value) and the value is a vector of clause addresses.
    pub index_table: HashMap<String, HashMap<Term, Vec<usize>>>,
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
            environment_stack: Vec::new(),
            index_table: HashMap::new(),
        }
    }

    // Registers an indexed clause for the given predicate.
    // The clause is associated with a key (typically the value of the first argument).
    pub fn register_indexed_clause(&mut self, predicate: String, key: Term, address: usize) {
        let entry = self.index_table.entry(predicate).or_insert_with(HashMap::new);
        entry.entry(key).or_insert_with(Vec::new).push(address);
    }

    // Registers a clause for the given predicate name.
    // Each call to this function adds one clause address.
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table
            .entry(name)
            .or_insert_with(Vec::new)
            .push(address);
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
                // Record the old binding on the trail.
                let prev = self.substitution.get(&name).cloned();
                self.trail.push(TrailEntry { variable: name.clone(), previous_value: prev });
                self.substitution.insert(name, other);
                true
            }
            (other, Term::Var(name)) => {
                let prev = self.substitution.get(&name).cloned();
                self.trail.push(TrailEntry { variable: name.clone(), previous_value: prev });
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
    //
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
                if let Some(clauses) = self.predicate_table.get(&predicate) {
                    if !clauses.is_empty() {
                        let mut alternatives = clauses.clone();
                        let jump_to = alternatives.remove(0);
                        let alternative_clauses = if alternatives.is_empty() {
                            None
                        } else {
                            Some(alternatives)
                        };
                        let cp = ChoicePoint {
                            saved_pc: self.pc,
                            saved_registers: self.registers.clone(),
                            saved_substitution: self.substitution.clone(),
                            saved_trail_len: self.trail.len(),
                            alternative_clauses,
                        };
                        self.choice_stack.push(cp);
                        println!("Calling predicate: {} using clause at address {}", predicate, jump_to);
                        self.pc = jump_to;
                    } else {
                        eprintln!("Call failed: no clause addresses for predicate {}", predicate);
                    }
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
                    alternative_clauses: None,
                };
                self.choice_stack.push(cp);
                println!("Choice point created.");
            },
            Instruction::Allocate { n } => {
                self.environment_stack.push(vec![None; n]);
                println!("Allocated environment of size {}", n);
            },
            Instruction::Deallocate => {
                if let Some(_) = self.environment_stack.pop() {
                    println!("Deallocated environment");
                } else {
                    eprintln!("Deallocate failed: no environment to deallocate");
                }
            },
            Instruction::ArithmeticIs { target, expression } => {
                let result = arithmetic::evaluate(&expression);
                if target < self.registers.len() {
                    self.registers[target] = Some(Term::Const(result));
                    println!("ArithmeticIs: evaluated expression to {} and stored in register {}", result, target);
                } else {
                    eprintln!("ArithmeticIs failed: target register {} out of bounds", target);
                }
            },
            Instruction::SetLocal { index, value } => {
                if let Some(env) = self.environment_stack.last_mut() {
                    if index < env.len() {
                        env[index] = Some(value);
                        println!("Set local variable at index {}", index);
                    } else {
                        eprintln!("SetLocal failed: index {} out of bounds", index);
                    }
                } else {
                    eprintln!("SetLocal failed: no environment allocated");
                }
            },
            Instruction::GetLocal { index, register } => {
                if let Some(env) = self.environment_stack.last() {
                    if index < env.len() {
                        if let Some(term) = env[index].clone() {
                            if register < self.registers.len() {
                                if let Some(reg_term) = self.registers[register].clone() {
                                    if !self.unify(&reg_term, &term) {
                                        eprintln!("GetLocal unification failed: cannot unify {:?} with {:?}", reg_term, term);
                                    }
                                } else {
                                    self.registers[register] = Some(term);
                                }
                            } else {
                                eprintln!("GetLocal failed: register {} out of bounds", register);
                            }
                        } else {
                            eprintln!("GetLocal failed: local variable at index {} is unbound", index);
                        }
                    } else {
                        eprintln!("GetLocal failed: index {} out of bounds", index);
                    }
                } else {
                    eprintln!("GetLocal failed: no environment allocated");
                }
            },
            Instruction::Fail => {
                if let Some(cp) = self.choice_stack.pop() {
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
            Instruction::IndexedCall { predicate, index_register } => {
                if index_register >= self.registers.len() {
                    eprintln!("IndexedCall failed: register {} out of bounds", index_register);
                } else if let Some(key_term) = self.registers[index_register].clone() {
                    if let Some(index_map) = self.index_table.get(&predicate) {
                        if let Some(clauses) = index_map.get(&key_term) {
                            if !clauses.is_empty() {
                                let mut alternatives = clauses.clone();
                                let jump_to = alternatives.remove(0);
                                let alternative_clauses = if alternatives.is_empty() {
                                    None
                                } else {
                                    Some(alternatives)
                                };
                                let cp = ChoicePoint {
                                    saved_pc: self.pc,
                                    saved_registers: self.registers.clone(),
                                    saved_substitution: self.substitution.clone(),
                                    saved_trail_len: self.trail.len(),
                                    alternative_clauses,
                                };
                                self.choice_stack.push(cp);
                                println!("IndexedCall: predicate {} with key {:?} jumping to clause at address {}", predicate, key_term, jump_to);
                                self.pc = jump_to;
                            } else {
                                eprintln!("IndexedCall failed: no clauses for predicate {} with key {:?}", predicate, key_term);
                            }
                        } else {
                            eprintln!("IndexedCall: no index entry for predicate {} with key {:?}", predicate, key_term);
                        }
                    } else {
                        eprintln!("IndexedCall failed: predicate {} not found in index table", predicate);
                    }
                } else {
                    eprintln!("IndexedCall failed: register {} is uninitialized", index_register);
                }
            },
            Instruction::TailCall { predicate } => {
                if let Some(_) = self.environment_stack.pop() {
                    println!("TailCall: deallocated current environment frame.");
                } else {
                    println!("TailCall: no environment frame to deallocate.");
                }
                if let Some(clauses) = self.predicate_table.get(&predicate) {
                    if !clauses.is_empty() {
                        let mut alternatives = clauses.clone();
                        let jump_to = alternatives.remove(0);
                        let alternative_clauses = if alternatives.is_empty() {
                            None
                        } else {
                            Some(alternatives)
                        };
                        let cp = ChoicePoint {
                            saved_pc: self.pc,
                            saved_registers: self.registers.clone(),
                            saved_substitution: self.substitution.clone(),
                            saved_trail_len: self.trail.len(),
                            alternative_clauses,
                        };
                        self.choice_stack.push(cp);
                        println!("TailCall: predicate {} tail-called to clause at address {}", predicate, jump_to);
                        self.pc = jump_to;
                    } else {
                        eprintln!("TailCall failed: no clause addresses for predicate {}", predicate);
                    }
                } else {
                    eprintln!("TailCall failed: predicate {} not found", predicate);
                }
            },
            Instruction::BuildCompound { target, functor, arg_registers } => {
                let mut args = Vec::new();
                for &reg in arg_registers.iter() {
                    if reg < self.registers.len() {
                        if let Some(term) = self.registers[reg].clone() {
                            args.push(term);
                        } else {
                            eprintln!("Error: Register {} is uninitialized", reg);
                            return true;
                        }
                    } else {
                        eprintln!("Error: Register {} out of bounds", reg);
                        return true;
                    }
                }
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