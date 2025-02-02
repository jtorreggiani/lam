use std::collections::HashMap;
use crate::term::Term;
use crate::arithmetic;

use super::{
    instruction::Instruction,
    frame::Frame,
    choice_point::ChoicePoint,
    trail::TrailEntry,
};

/// The set of errors that can occur during execution of the LAM.
#[derive(Debug)]
pub enum MachineError {
    RegisterOutOfBounds(usize),
    UninitializedRegister(usize),
    UnificationFailed(String),
    EnvironmentMissing,
    PredicateNotFound(String),
    PredicateClauseNotFound(String),
    NoChoicePoint,
    StructureMismatch {
        expected_functor: String,
        expected_arity: usize,
        found_functor: String,
        found_arity: usize,
    },
    NotACompoundTerm(usize),
    NoIndexedClause(String, Term),
    NoIndexEntry(String, Term),
    PredicateNotInIndex(String),
    NoMoreInstructions,
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
    /// Substitution environment mapping variable unique id to Terms.
    pub substitution: HashMap<usize, Term>,
    /// A control stack to hold frames for predicate calls.
    pub control_stack: Vec<Frame>,
    /// A predicate table mapping predicate names to lists of clause addresses.
    pub predicate_table: HashMap<String, Vec<usize>>,
    /// A stack to hold choice points for backtracking.
    pub choice_stack: Vec<ChoicePoint>,
    /// A trail to record variable bindings (for backtracking).
    pub trail: Vec<TrailEntry>,
    /// A stack of environment frames for local variables.
    pub environment_stack: Vec<Vec<Option<Term>>>,
    /// Index table mapping predicate names to an index mapping.
    pub index_table: HashMap<String, HashMap<Term, Vec<usize>>>,
    /// Mapping from variable unique id to its original string name.
    pub variable_names: HashMap<usize, String>,
}

impl Machine {
    /// Create a new machine with a specified number of registers and given code.
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
            variable_names: HashMap::new(),
        }
    }

    /// Helper function to bind a variable (by its unique id) to a term.
    /// Returns false if the occurs check fails; otherwise, returns true after performing the binding.
    fn bind_variable(&mut self, var: &usize, term: &Term) -> bool {
        if self.occurs_check(var, term) {
            return false;
        }
        let prev = self.substitution.get(var).cloned();
        self.trail.push(TrailEntry { variable: *var, previous_value: prev });
        self.substitution.insert(*var, term.clone());
        true
    }

    /// Returns true if `var` occurs anywhere inside `term`.
    fn occurs_check(&self, var: &usize, term: &Term) -> bool {
        match term {
            Term::Var(v) => v == var,
            Term::Const(_) => false,
            Term::Compound(_, args) => args.iter().any(|arg| self.occurs_check(var, arg)),
            Term::Lambda(param, body) => {
                if param == var {
                    false
                } else {
                    self.occurs_check(var, body)
                }
            },
            Term::App(fun, arg) => self.occurs_check(var, fun) || self.occurs_check(var, arg),
        }
    }

    /// Registers an indexed clause for the given predicate.
    pub fn register_indexed_clause(&mut self, predicate: String, key: Term, address: usize) {
        let entry = self.index_table.entry(predicate).or_insert_with(HashMap::new);
        entry.entry(key).or_insert_with(Vec::new).push(address);
    }

    /// Registers a clause for the given predicate name.
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table
            .entry(name)
            .or_insert_with(Vec::new)
            .push(address);
    }

    /// Resolve a term to its current bound value, if any.
    fn resolve(&self, term: &Term) -> Term {
        match term {
            Term::Var(v) => {
                if let Some(bound) = self.substitution.get(v) {
                    self.resolve(bound)
                } else {
                    term.clone()
                }
            }
            _ => term.clone(),
        }
    }

    /// Unify two terms.
    /// If unification fails, returns false.
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> bool {
        let term1 = self.resolve(t1);
        let term2 = self.resolve(t2);

        if term1 == term2 {
            return true;
        }

        match (term1, term2) {
            (Term::Const(a), Term::Const(b)) => a == b,
            (Term::Var(ref v), ref other) => self.bind_variable(v, other),
            (ref other, Term::Var(ref v)) => self.bind_variable(v, other),
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
            },
            _ => false,
        }
    }

    /// Execute one instruction.
    /// Returns `Ok(())` if the instruction executed normally,
    /// or an appropriate `MachineError` if an error occurred.
    pub fn step(&mut self) -> Result<(), MachineError> {
        if self.pc >= self.code.len() {
            return Err(MachineError::NoMoreInstructions);
        }
        let instr = self.code[self.pc].clone();
        self.pc += 1;
        match instr {
            Instruction::PutConst { register, value } => {
                if register < self.registers.len() {
                    self.registers[register] = Some(Term::Const(value));
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(register))
                }
            },
            Instruction::PutVar { register, var_id, name } => {
                if register < self.registers.len() {
                    self.registers[register] = Some(Term::Var(var_id));
                    self.variable_names.insert(var_id, name);
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(register))
                }
            },
            Instruction::GetConst { register, value } => {
                if register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(register));
                }
                if let Some(term) = self.registers[register].clone() {
                    let goal = Term::Const(value);
                    if !self.unify(&term, &goal) {
                        return Err(MachineError::UnificationFailed(format!(
                            "Cannot unify {:?} with {:?}",
                            term, goal
                        )));
                    }
                    Ok(())
                } else {
                    Err(MachineError::UninitializedRegister(register))
                }
            },
            Instruction::GetVar { register, var_id, name } => {
                if register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(register));
                }
                // Ensure the machine knows the name for this variable.
                self.variable_names.entry(var_id).or_insert(name);
                if let Some(term) = self.registers[register].clone() {
                    let goal = Term::Var(var_id);
                    if !self.unify(&goal, &term) {
                        return Err(MachineError::UnificationFailed(format!(
                            "Cannot unify {:?} with {:?}",
                            goal, term
                        )));
                    }
                    Ok(())
                } else {
                    // If uninitialized, simply set the register to the variable.
                    self.registers[register] = Some(Term::Var(var_id));
                    Ok(())
                }
            },
            Instruction::Call { predicate } => {
                if let Some(clauses) = self.predicate_table.get(&predicate) {
                    if clauses.is_empty() {
                        return Err(MachineError::PredicateClauseNotFound(predicate));
                    }
                    self.control_stack.push(Frame {
                        return_pc: self.pc,
                    });
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
                    self.pc = jump_to;
                    Ok(())
                } else {
                    Err(MachineError::PredicateNotFound(predicate))
                }
            },
            Instruction::Proceed => {
                if let Some(frame) = self.control_stack.pop() {
                    self.pc = frame.return_pc;
                }
                Ok(())
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
                Ok(())
            },
            Instruction::Allocate { n } => {
                self.environment_stack.push(vec![None; n]);
                Ok(())
            },
            Instruction::Deallocate => {
                if self.environment_stack.pop().is_some() {
                    Ok(())
                } else {
                    Err(MachineError::EnvironmentMissing)
                }
            },
            Instruction::ArithmeticIs { target, expression } => {
                let result = arithmetic::evaluate(&expression);
                if target < self.registers.len() {
                    self.registers[target] = Some(Term::Const(result));
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(target))
                }
            },
            Instruction::SetLocal { index, value } => {
                if let Some(env) = self.environment_stack.last_mut() {
                    if index < env.len() {
                        env[index] = Some(value);
                        Ok(())
                    } else {
                        Err(MachineError::RegisterOutOfBounds(index))
                    }
                } else {
                    Err(MachineError::EnvironmentMissing)
                }
            },
            Instruction::GetLocal { index, register } => {
                if let Some(env) = self.environment_stack.last() {
                    if index < env.len() {
                        if let Some(term) = env[index].clone() {
                            if register < self.registers.len() {
                                if let Some(reg_term) = self.registers[register].clone() {
                                    if !self.unify(&reg_term, &term) {
                                        return Err(MachineError::UnificationFailed(format!(
                                            "Cannot unify {:?} with {:?}",
                                            reg_term, term
                                        )));
                                    }
                                } else {
                                    self.registers[register] = Some(term);
                                }
                                Ok(())
                            } else {
                                Err(MachineError::RegisterOutOfBounds(register))
                            }
                        } else {
                            Err(MachineError::UninitializedRegister(index))
                        }
                    } else {
                        Err(MachineError::RegisterOutOfBounds(index))
                    }
                } else {
                    Err(MachineError::EnvironmentMissing)
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
                    Ok(())
                } else {
                    Err(MachineError::NoChoicePoint)
                }
            },
            Instruction::GetStructure { register, functor, arity } => {
                if register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(register));
                }
                if let Some(term) = self.registers[register].clone() {
                    match term {
                        Term::Compound(ref f, ref args) => {
                            if f == &functor && args.len() == arity {
                                Ok(())
                            } else {
                                Err(MachineError::StructureMismatch {
                                    expected_functor: functor,
                                    expected_arity: arity,
                                    found_functor: f.clone(),
                                    found_arity: args.len(),
                                })
                            }
                        },
                        _ => Err(MachineError::NotACompoundTerm(register)),
                    }
                } else {
                    Err(MachineError::UninitializedRegister(register))
                }
            },
            Instruction::IndexedCall { predicate, index_register } => {
                if index_register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(index_register));
                }
                if let Some(key_term) = self.registers[index_register].clone() {
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
                                self.pc = jump_to;
                                Ok(())
                            } else {
                                Err(MachineError::NoIndexedClause(predicate, key_term))
                            }
                        } else {
                            Err(MachineError::NoIndexEntry(predicate, key_term))
                        }
                    } else {
                        Err(MachineError::PredicateNotInIndex(predicate))
                    }
                } else {
                    Err(MachineError::UninitializedRegister(index_register))
                }
            },
            Instruction::TailCall { predicate } => {
                let _ = self.environment_stack.pop(); // Deallocate current environment frame.
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
                        self.pc = jump_to;
                        Ok(())
                    } else {
                        Err(MachineError::PredicateClauseNotFound(predicate))
                    }
                } else {
                    Err(MachineError::PredicateNotFound(predicate))
                }
            },
            Instruction::AssertClause { predicate, address } => {
                self.predicate_table
                    .entry(predicate.clone())
                    .or_insert_with(Vec::new)
                    .push(address);
                Ok(())
            },
            Instruction::RetractClause { predicate, address } => {
                if let Some(clauses) = self.predicate_table.get_mut(&predicate) {
                    if let Some(pos) = clauses.iter().position(|&a| a == address) {
                        clauses.remove(pos);
                        Ok(())
                    } else {
                        Err(MachineError::PredicateClauseNotFound(predicate))
                    }
                } else {
                    Err(MachineError::PredicateNotFound(predicate))
                }
            },
            Instruction::Cut => {
                self.choice_stack.clear();
                Ok(())
            },
            Instruction::BuildCompound { target, functor, arg_registers } => {
                let mut args = Vec::new();
                for &reg in arg_registers.iter() {
                    if reg >= self.registers.len() {
                        return Err(MachineError::RegisterOutOfBounds(reg));
                    }
                    if let Some(term) = self.registers[reg].clone() {
                        args.push(term);
                    } else {
                        return Err(MachineError::UninitializedRegister(reg));
                    }
                }
                if target < self.registers.len() {
                    self.registers[target] = Some(Term::Compound(functor, args));
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(target))
                }
            },
        }
    }

    /// Run the machine until no more instructions are available.
    /// Returns `Ok(())` if execution completed, or a `MachineError` otherwise.
    pub fn run(&mut self) -> Result<(), MachineError> {
        while self.pc < self.code.len() {
            self.step()?;
        }
        Ok(())
    }
}
