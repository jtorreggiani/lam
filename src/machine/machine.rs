use std::collections::HashMap;
use crate::term::Term;
use crate::union_find::UnionFind;
use crate::arithmetic; // restore usage

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
    /// Union-find data structure for indexing.
    pub uf: UnionFind,
}

impl Machine {
    /// Create a new machine with a specified number of registers and given code.
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
            choice_stack: Vec::new(),
            control_stack: Vec::new(),
            environment_stack: Vec::new(),
            index_table: HashMap::new(),
            predicate_table: HashMap::new(),
            substitution: HashMap::new(),
            trail: Vec::new(),
            uf: UnionFind::new(),
            variable_names: HashMap::new(),
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

    /// Unify two terms.
    /// If unification fails, returns an error.
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> Result<(), MachineError> {
        let resolved1 = self.uf.resolve(t1);
        let resolved2 = self.uf.resolve(t2);

        match (&resolved1, &resolved2) {
            (Term::Const(a), Term::Const(b)) => {
                if a == b {
                    Ok(())
                } else {
                    Err(MachineError::UnificationFailed(format!(
                        "Constants do not match: {} vs {}",
                        a, b
                    )))
                }
            },
            (Term::Var(v), other) => {
                self.uf.bind(*v, other)
            },
            (other, Term::Var(v)) => {
                self.uf.bind(*v, other)
            },
            (Term::Compound(f1, args1), Term::Compound(f2, args2)) => {
                if f1 != f2 || args1.len() != args2.len() {
                    return Err(MachineError::UnificationFailed(format!(
                        "Compound term mismatch: {} vs {}",
                        f1, f2
                    )));
                }
                for (a, b) in args1.iter().zip(args2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            },
            (t1, t2) => Err(MachineError::UnificationFailed(format!(
                "Failed to unify {:?} with {:?}",
                t1, t2
            ))),
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
                self.exec_put_const(register, value)
            },
            Instruction::PutVar { register, var_id, name } => {
                self.exec_put_var(register, var_id, name)
            },
            Instruction::GetConst { register, value } => {
                self.exec_get_const(register, value)
            },
            Instruction::GetVar { register, var_id, name } => {
                self.exec_get_var(register, var_id, name)
            },
            Instruction::Call { predicate } => {
                self.exec_call(predicate)
            },
            Instruction::Proceed => {
                self.exec_proceed()
            },
            Instruction::Choice { alternative } => {
                self.exec_choice(alternative)
            },
            Instruction::Allocate { n } => {
                self.exec_allocate(n)
            },
            Instruction::Deallocate => {
                self.exec_deallocate()
            },
            Instruction::ArithmeticIs { target, expression } => {
                self.exec_arithmetic_is(target, expression)
            },
            Instruction::SetLocal { index, value } => {
                self.exec_set_local(index, value)
            },
            Instruction::GetLocal { index, register } => {
                self.exec_get_local(index, register)
            },
            Instruction::Fail => {
                self.exec_fail()
            },
            Instruction::GetStructure { register, functor, arity } => {
                self.exec_get_structure(register, functor, arity)
            },
            Instruction::IndexedCall { predicate, index_register } => {
                self.exec_indexed_call(predicate, index_register)
            },
            Instruction::TailCall { predicate } => {
                self.exec_tail_call(predicate)
            },
            Instruction::AssertClause { predicate, address } => {
                self.exec_assert_clause(predicate, address)
            },
            Instruction::RetractClause { predicate, address } => {
                self.exec_retract_clause(predicate, address)
            },
            Instruction::Cut => {
                self.exec_cut()
            },
            Instruction::BuildCompound { target, functor, arg_registers } => {
                self.exec_build_compound(target, functor, arg_registers)
            },
        }
    }

    fn exec_put_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        if register < self.registers.len() {
            self.registers[register] = Some(Term::Const(value));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    fn exec_put_var(&mut self, register: usize, var_id: usize, name: String) -> Result<(), MachineError> {
        if register < self.registers.len() {
            self.registers[register] = Some(Term::Var(var_id));
            self.variable_names.insert(var_id, name);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    fn exec_get_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        if register >= self.registers.len() {
            return Err(MachineError::RegisterOutOfBounds(register));
        }
        if let Some(term) = self.registers[register].clone() {
            self.unify(&term, &Term::Const(value)).map_err(|_| {
                MachineError::UnificationFailed(format!(
                    "Cannot unify {:?} with {:?}",
                    term, Term::Const(value)
                ))
            })?;
            Ok(())
        } else {
            Err(MachineError::UninitializedRegister(register))
        }
    }

    fn exec_get_var(
        &mut self,
        register: usize,
        var_id: usize,
        name: String,
    ) -> Result<(), MachineError> {
        if register >= self.registers.len() {
            return Err(MachineError::RegisterOutOfBounds(register));
        }
        // Ensure the machine knows the name for this variable.
        self.variable_names.entry(var_id).or_insert(name);
        if let Some(term) = self.registers[register].clone() {
            let goal = Term::Var(var_id);
            self.unify(&goal, &term).map_err(|_| {
                MachineError::UnificationFailed(format!(
                    "Cannot unify {:?} with {:?}", goal, term
                ))
            })?;
            Ok(())
        } else {
            // If uninitialized, simply set the register to the variable.
            self.registers[register] = Some(Term::Var(var_id));
            Ok(())
        }
    }

    fn exec_call(&mut self, predicate: String) -> Result<(), MachineError> {
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
                saved_control_stack: self.control_stack.clone(),
                alternative_clauses,
                saved_uf: self.uf.clone(), // Save the union–find state.
            };
            self.choice_stack.push(cp);
            self.pc = jump_to;
            Ok(())
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    fn exec_proceed(&mut self) -> Result<(), MachineError> {
        if let Some(frame) = self.control_stack.pop() {
            self.pc = frame.return_pc;
        }
        Ok(())
    }

    fn exec_choice(&mut self, alternative: usize) -> Result<(), MachineError> {
        let cp = ChoicePoint {
            saved_pc: self.pc,
            saved_registers: self.registers.clone(),
            saved_substitution: self.substitution.clone(),
            saved_trail_len: self.trail.len(),
            saved_control_stack: self.control_stack.clone(),
            alternative_clauses: Some(vec![alternative]),
            saved_uf: self.uf.clone(), // Save the union–find state.
        };
        self.choice_stack.push(cp);
        Ok(())
    }

    fn exec_allocate(&mut self, n: usize) -> Result<(), MachineError> {
        self.environment_stack.push(vec![None; n]);
        Ok(())
    }

    fn exec_deallocate(&mut self) -> Result<(), MachineError> {
        if self.environment_stack.pop().is_some() {
            Ok(())
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    fn exec_arithmetic_is(
        &mut self,
        target: usize,
        expression: arithmetic::Expression,
    ) -> Result<(), MachineError> {
        let result = arithmetic::evaluate(&expression);
        if target < self.registers.len() {
            self.registers[target] = Some(Term::Const(result));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }

    fn exec_set_local(&mut self, index: usize, value: Term) -> Result<(), MachineError> {
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
    }

    fn exec_get_local(&mut self, index: usize, register: usize) -> Result<(), MachineError> {
        if let Some(env) = self.environment_stack.last() {
            if index < env.len() {
                if let Some(term) = env[index].clone() {
                    if register < self.registers.len() {
                        if let Some(reg_term) = self.registers[register].clone() {
                            self.unify(&reg_term, &term).map_err(|_|{
                                MachineError::UnificationFailed(format!(
                                    "Cannot unify {:?} with {:?}",
                                    reg_term, term
                                ))
                            })?;
                            Ok(())
                        } else {
                            self.registers[register] = Some(term);
                            Ok(())
                        }
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
    }

    fn exec_fail(&mut self) -> Result<(), MachineError> {
        // Attempt to backtrack by checking available choice points.
        while let Some(mut cp) = self.choice_stack.pop() {
            // Restore trail to saved length.
            while self.trail.len() > cp.saved_trail_len {
                if let Some(entry) = self.trail.pop() {
                    match entry.previous_value {
                        Some(prev_val) => {
                            self.substitution.insert(entry.variable, prev_val);
                        },
                        None => {
                            self.substitution.remove(&entry.variable);
                        }
                    }
                }
            }
            // Restore registers, substitution, control stack, and union–find state.
            self.registers = cp.saved_registers.clone();
            self.substitution = cp.saved_substitution.clone();
            self.control_stack = cp.saved_control_stack.clone();
            self.uf = cp.saved_uf.clone();

            if let Some(ref mut alternatives) = cp.alternative_clauses {
                if let Some(next_addr) = alternatives.pop() {
                    if !alternatives.is_empty() {
                        self.choice_stack.push(cp);
                    }
                    self.pc = next_addr;
                    return Ok(());
                }
            }
        }
        // If no choice point with alternatives is found, signal failure.
        Err(MachineError::NoChoicePoint)
    }

    fn exec_get_structure(
        &mut self,
        register: usize,
        functor: String,
        arity: usize
    ) -> Result<(), MachineError> {
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
    }

    fn exec_indexed_call(
        &mut self,
        predicate: String,
        index_register: usize
    ) -> Result<(), MachineError> {
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
                            saved_control_stack: self.control_stack.clone(),
                            alternative_clauses,
                            saved_uf: self.uf.clone(), // Save the union–find state.
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
    }

    fn exec_tail_call(&mut self, predicate: String) -> Result<(), MachineError> {
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
                    saved_control_stack: self.control_stack.clone(),
                    alternative_clauses,
                    saved_uf: self.uf.clone(), // Save the union–find state.
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
    }

    fn exec_assert_clause(
        &mut self,
        predicate: String,
        address: usize
    ) -> Result<(), MachineError> {
        self.predicate_table
            .entry(predicate.clone())
            .or_insert_with(Vec::new)
            .push(address);
        Ok(())
    }

    fn exec_retract_clause(
        &mut self,
        predicate: String,
        address: usize
    ) -> Result<(), MachineError> {
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
    }

    fn exec_cut(&mut self) -> Result<(), MachineError> {
        self.choice_stack.clear();
        Ok(())
    }

    fn exec_build_compound(
        &mut self,
        target: usize,
        functor: String,
        arg_registers: Vec<usize>
    ) -> Result<(), MachineError> {
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
