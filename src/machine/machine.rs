use std::collections::HashMap;
use crate::term::Term;
use crate::union_find::UnionFind;
use log::{debug, error};
use thiserror::Error;

use crate::arithmetic;
use super::{
    instruction::Instruction,
    frame::Frame,
    choice_point::ChoicePoint,
    // Note: trail module is no longer used.
};

/// The set of errors that can occur during execution of the LAM.
#[derive(Debug, Error)]
pub enum MachineError {
    #[error("Register {0} is out of bounds.")]
    RegisterOutOfBounds(usize),
    #[error("Register {0} is uninitialized.")]
    UninitializedRegister(usize),
    #[error("Unification failed: {0}")]
    UnificationFailed(String),
    #[error("Environment missing.")]
    EnvironmentMissing,
    #[error("Predicate not found: {0}")]
    PredicateNotFound(String),
    #[error("Predicate clause not found: {0}")]
    PredicateClauseNotFound(String),
    #[error("No choice point available.")]
    NoChoicePoint,
    #[error("Structure mismatch: expected {expected_functor}/{expected_arity} but found {found_functor}/{found_arity}.")]
    StructureMismatch {
        expected_functor: String,
        expected_arity: usize,
        found_functor: String,
        found_arity: usize,
    },
    #[error("Term in register {0} is not a compound term.")]
    NotACompoundTerm(usize),
    #[error("No indexed clause for predicate {0} with key {1:?}.")]
    NoIndexedClause(String, crate::term::Term),
    #[error("No index entry for predicate {0} with key {1:?}.")]
    NoIndexEntry(String, crate::term::Term),
    #[error("Predicate {0} is not in the index.")]
    PredicateNotInIndex(String),
    #[error("No more instructions.")]
    NoMoreInstructions,
}

/// Built-in predicate type.
pub type BuiltinPredicate = fn(&mut Machine) -> Result<(), MachineError>;

/// The abstract machine structure.
#[derive(Debug)]
pub struct Machine {
    pub registers: Vec<Option<Term>>,
    pub code: Vec<Instruction>,
    pub pc: usize,
    pub substitution: HashMap<usize, Term>,
    pub control_stack: Vec<Frame>,
    pub predicate_table: HashMap<String, Vec<usize>>,
    pub choice_stack: Vec<ChoicePoint>,
    // Removed: trail field is no longer needed.
    pub environment_stack: Vec<Vec<Option<Term>>>,
    // Updated: index_table now uses a vector of terms as the key for multi-argument indexing.
    pub index_table: HashMap<String, HashMap<Vec<Term>, Vec<usize>>>,
    pub variable_names: HashMap<usize, String>,
    pub uf: UnionFind,
    /// If true, print detailed execution trace.
    pub verbose: bool,
    /// Built-in predicates.
    pub builtins: HashMap<String, BuiltinPredicate>,
}

impl Machine {
    #[inline(always)]
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        let mut machine = Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
            choice_stack: Vec::new(),
            control_stack: Vec::new(),
            environment_stack: Vec::new(),
            index_table: HashMap::new(),
            predicate_table: HashMap::new(),
            substitution: HashMap::new(),
            // Removed: trail field.
            uf: UnionFind::new(),
            variable_names: HashMap::new(),
            verbose: false,
            builtins: HashMap::new(),
        };
        // Register an example built-in predicate "print"
        machine.builtins.insert("print".to_string(), Machine::builtin_print);
        machine.builtins.insert("print_subst".to_string(), Machine::builtin_print_subst);
        machine
    }

    /// Built-in predicate example: prints the current registers.
    #[inline(always)]
    fn builtin_print(&mut self) -> Result<(), MachineError> {
        println!("--- Machine Registers ---");
        // Print only registers that contain a value.
        for (i, reg) in self.registers.iter().enumerate() {
            if let Some(term) = reg {
                match term {
                    Term::Var(id) => {
                        // Clone the variable name so that we have an owned String.
                        let name = self.variable_names.get(id).cloned().unwrap_or_default();
                        println!("Reg {:>3}: Var({}) \"{}\"", i, id, name);
                    }
                    _ => println!("Reg {:>3}: {:?}", i, term),
                }
            }
        }
        println!("-------------------------");
        Ok(())
    }

    /// Built-in predicate that prints the current substitution in a human-readable format.
    #[inline(always)]
    fn builtin_print_subst(&mut self) -> Result<(), MachineError> {
        println!("--- Current Substitution ---");
        if self.substitution.is_empty() {
            println!("(no bindings)");
        } else {
            for (var_id, term) in &self.substitution {
                // Look up the variable's name if available.
                let var_name = self.variable_names.get(var_id).cloned().unwrap_or_default();
                println!("Variable {} (id {}) = {:?}", if var_name.is_empty() { format!("_{}", var_id) } else { var_name }, var_id, term);
            }
        }
        println!("----------------------------");
        Ok(())
    }

    /// Debug helper: if verbose, print the current PC and instruction.
    #[inline(always)]
    fn trace(&self, instr: &Instruction) {
        if self.verbose {
            debug!("PC {}: {:?}", self.pc - 1, instr);
            debug!("Registers: {:?}", self.registers);
        }
    }    

    /// Helper to update the index table when retracting a clause.
    #[inline(always)]
    fn update_index_table_on_retract(&mut self, predicate: &str, clause_address: usize) {
        if let Some(index_map) = self.index_table.get_mut(predicate) {
            // Iterate over keys and remove the clause_address if found.
            for (_key, clauses) in index_map.iter_mut() {
                if let Some(pos) = clauses.iter().position(|&a| a == clause_address) {
                    clauses.remove(pos);
                }
            }
        }
    }

    /// Registers an indexed clause for the given predicate.
    /// Now accepts a key as a vector of terms for multi-argument indexing.
    #[inline(always)]
    pub fn register_indexed_clause(&mut self, predicate: String, key: Vec<Term>, address: usize) {
        let entry = self.index_table.entry(predicate).or_insert_with(HashMap::new);
        entry.entry(key).or_insert_with(Vec::new).push(address);
    }

    /// Registers a clause for the given predicate name.
    #[inline(always)]
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table
            .entry(name)
            .or_insert_with(Vec::new)
            .push(address);
    }

    /// Unify two terms.
    #[inline(always)]
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> Result<(), MachineError> {
        let resolved1 = self.uf.resolve(t1);
        let resolved2 = self.uf.resolve(t2);

        match (&resolved1, &resolved2) {
            (Term::Const(a), Term::Const(b)) => {
                if a == b { Ok(()) }
                else {
                    Err(MachineError::UnificationFailed(format!(
                        "Constants do not match: {} vs {}", a, b
                    )))
                }
            },
            (Term::Var(v), other) => self.uf.bind(*v, other),
            (other, Term::Var(v)) => self.uf.bind(*v, other),
            (Term::Compound(f1, args1), Term::Compound(f2, args2)) => {
                if f1 != f2 || args1.len() != args2.len() {
                    return Err(MachineError::UnificationFailed(format!(
                        "Compound term mismatch: {} vs {}", f1, f2
                    )));
                }
                for (a, b) in args1.iter().zip(args2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            },
            (t1, t2) => Err(MachineError::UnificationFailed(format!(
                "Failed to unify {:?} with {:?}", t1, t2
            ))),
        }
    }

    /// Execute one instruction.
    #[inline(always)]
    pub fn step(&mut self) -> Result<(), MachineError> {
        let instr = self.code.get(self.pc)
            .ok_or(MachineError::NoMoreInstructions)?
            .clone();
        self.pc += 1;
        self.trace(&instr);
        match instr {
            Instruction::PutConst { register, value } => self.exec_put_const(register, value),
            Instruction::PutVar { register, var_id, name } => self.exec_put_var(register, var_id, name),
            Instruction::GetConst { register, value } => self.exec_get_const(register, value),
            Instruction::GetVar { register, var_id, name } => self.exec_get_var(register, var_id, name),
            Instruction::Call { predicate } => self.exec_call(predicate),
            Instruction::Proceed => self.exec_proceed(),
            Instruction::Choice { alternative } => self.exec_choice(alternative),
            Instruction::Allocate { n } => self.exec_allocate(n),
            Instruction::Deallocate => self.exec_deallocate(),
            Instruction::ArithmeticIs { target, expression } => self.exec_arithmetic_is(target, expression),
            Instruction::SetLocal { index, value } => self.exec_set_local(index, value),
            Instruction::GetLocal { index, register } => self.exec_get_local(index, register),
            Instruction::Fail => self.exec_fail(),
            Instruction::GetStructure { register, functor, arity } => self.exec_get_structure(register, functor, arity),
            Instruction::IndexedCall { predicate, index_register } => self.exec_indexed_call(predicate, index_register),
            Instruction::MultiIndexedCall { predicate, index_registers } => self.exec_multi_indexed_call(predicate, index_registers),
            Instruction::TailCall { predicate } => self.exec_tail_call(predicate),
            Instruction::AssertClause { predicate, address } => self.exec_assert_clause(predicate, address),
            Instruction::RetractClause { predicate, address } => self.exec_retract_clause(predicate, address),
            Instruction::Cut => self.exec_cut(),
            Instruction::BuildCompound { target, functor, arg_registers } => self.exec_build_compound(target, functor, arg_registers),
        }
    }

    #[inline(always)]
    fn exec_put_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Const(value));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    #[inline(always)]
    fn exec_put_var(&mut self, register: usize, var_id: usize, name: String) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Var(var_id));
            self.variable_names.insert(var_id, name);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    #[inline(always)]
    fn exec_get_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        match self.registers.get(register) {
            Some(Some(term)) => {
                let term_clone = term.clone(); // break the immutable borrow
                self.unify(&term_clone, &Term::Const(value)).map_err(|_| {
                    MachineError::UnificationFailed(format!(
                        "Cannot unify {:?} with {:?}", term_clone, Term::Const(value)
                    ))
                })
            },
            Some(None) => Err(MachineError::UninitializedRegister(register)),
            None => Err(MachineError::RegisterOutOfBounds(register)),
        }
    }

    #[inline(always)]
    fn exec_get_var(&mut self, register: usize, var_id: usize, name: String) -> Result<(), MachineError> {
        if register >= self.registers.len() {
            return Err(MachineError::RegisterOutOfBounds(register));
        }
        self.variable_names.entry(var_id).or_insert(name);
        if let Some(term) = self.registers[register].clone() {
            let goal = Term::Var(var_id);
            self.unify(&goal, &term).map_err(|_| {
                MachineError::UnificationFailed(format!("Cannot unify {:?} with {:?}", goal, term))
            })
        } else {
            self.registers[register] = Some(Term::Var(var_id));
            Ok(())
        }
    }

    /// Executes a call: checks built-ins first.
    #[inline(always)]
    fn exec_call(&mut self, predicate: String) -> Result<(), MachineError> {
        if let Some(builtin) = self.builtins.get(&predicate) {
            // Execute built-in predicate.
            builtin(self)
        } else if let Some(clauses) = self.predicate_table.get(&predicate) {
            if clauses.is_empty() {
                return Err(MachineError::PredicateClauseNotFound(predicate));
            }
            self.control_stack.push(Frame { return_pc: self.pc });
            let mut alternatives = clauses.clone();
            let jump_to = alternatives.remove(0);
            let alternative_clauses = if alternatives.is_empty() { None } else { Some(alternatives) };
            let cp = ChoicePoint {
                saved_pc: self.pc,
                saved_registers: self.registers.clone(),
                saved_substitution: self.substitution.clone(),
                saved_control_stack: self.control_stack.clone(),
                alternative_clauses,
                saved_uf: self.uf.clone(),
            };
            self.choice_stack.push(cp);
            self.pc = jump_to;
            Ok(())
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    #[inline(always)]
    fn exec_proceed(&mut self) -> Result<(), MachineError> {
        if let Some(frame) = self.control_stack.pop() {
            self.pc = frame.return_pc;
        }
        Ok(())
    }

    #[inline(always)]
    fn exec_choice(&mut self, alternative: usize) -> Result<(), MachineError> {
        let cp = ChoicePoint {
            saved_pc: self.pc,
            saved_registers: self.registers.clone(),
            saved_substitution: self.substitution.clone(),
            saved_control_stack: self.control_stack.clone(),
            alternative_clauses: Some(vec![alternative]),
            saved_uf: self.uf.clone(),
        };
        self.choice_stack.push(cp);
        Ok(())
    }

    #[inline(always)]
    fn exec_allocate(&mut self, n: usize) -> Result<(), MachineError> {
        self.environment_stack.push(vec![None; n]);
        Ok(())
    }

    #[inline(always)]
    fn exec_deallocate(&mut self) -> Result<(), MachineError> {
        if self.environment_stack.pop().is_some() {
            Ok(())
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    #[inline(always)]
    fn exec_arithmetic_is(&mut self, target: usize, expression: arithmetic::Expression) -> Result<(), MachineError> {
        let result = arithmetic::evaluate(&expression, &self.registers)?;
        if let Some(slot) = self.registers.get_mut(target) {
            *slot = Some(Term::Const(result));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }

    #[inline(always)]
    fn exec_set_local(&mut self, index: usize, value: Term) -> Result<(), MachineError> {
        if let Some(env) = self.environment_stack.last_mut() {
            if let Some(slot) = env.get_mut(index) {
                *slot = Some(value);
                Ok(())
            } else {
                Err(MachineError::RegisterOutOfBounds(index))
            }
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    #[inline(always)]
    fn exec_get_local(&mut self, index: usize, register: usize) -> Result<(), MachineError> {
        if let Some(env) = self.environment_stack.last() {
            // First, obtain a clone of the term in the environment.
            let term = env.get(index)
                .and_then(|t| t.clone())
                .ok_or(MachineError::UninitializedRegister(index))?;
            // Then get a mutable reference to the register slot.
            if let Some(reg_slot) = self.registers.get_mut(register) {
                match reg_slot {
                    Some(existing_term) => {
                        let cloned = existing_term.clone();
                        self.unify(&cloned, &term).map_err(|_| {
                            MachineError::UnificationFailed(format!("Cannot unify {:?} with {:?}", cloned, term))
                        })
                    },
                    None => {
                        *reg_slot = Some(term);
                        Ok(())
                    }
                }
            } else {
                Err(MachineError::RegisterOutOfBounds(register))
            }
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    /// Executes a failure and triggers backtracking.
    /// 
    /// **Note:** The trail mechanism has been removed in favor of rolling back the union-find state.
    #[inline(always)]
    fn exec_fail(&mut self) -> Result<(), MachineError> {
        while let Some(cp) = self.choice_stack.pop() {
            // Instead of cloning saved state, use it directly.
            self.registers = cp.saved_registers;
            self.substitution = cp.saved_substitution;
            self.control_stack = cp.saved_control_stack;
            self.uf = cp.saved_uf;

            if let Some(mut alternatives) = cp.alternative_clauses {
                if let Some(next_addr) = alternatives.pop() {
                    if !alternatives.is_empty() {
                        // Push back the choice point with remaining alternatives.
                        self.choice_stack.push(ChoicePoint {
                            saved_pc: cp.saved_pc,
                            saved_registers: self.registers.clone(),
                            saved_substitution: self.substitution.clone(),
                            saved_control_stack: self.control_stack.clone(),
                            alternative_clauses: Some(alternatives),
                            saved_uf: self.uf.clone(),
                        });
                    }
                    self.pc = next_addr;
                    return Ok(());
                }
            }
        }
        Err(MachineError::NoChoicePoint)
    }

    #[inline(always)]
    fn exec_get_structure(&mut self, register: usize, functor: String, arity: usize) -> Result<(), MachineError> {
        match self.registers.get(register).and_then(|t| t.clone()) {
            Some(Term::Compound(ref f, ref args)) => {
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
            Some(_) => Err(MachineError::NotACompoundTerm(register)),
            None => Err(MachineError::UninitializedRegister(register)),
        }
    }

    #[inline(always)]
    fn exec_indexed_call(&mut self, predicate: String, index_register: usize) -> Result<(), MachineError> {
        let key_term = self.registers.get(index_register)
            .ok_or(MachineError::RegisterOutOfBounds(index_register))?
            .clone()
            .ok_or(MachineError::UninitializedRegister(index_register))?;
        // For backward compatibility, treat single-key indexing as a vector with one element.
        let key_vec = vec![key_term];
        if let Some(index_map) = self.index_table.get(&predicate) {
            if let Some(clauses) = index_map.get(&key_vec) {
                if !clauses.is_empty() {
                    let mut alternatives = clauses.clone();
                    let jump_to = alternatives.remove(0);
                    let alternative_clauses = if alternatives.is_empty() { None } else { Some(alternatives) };
                    let cp = ChoicePoint {
                        saved_pc: self.pc,
                        saved_registers: self.registers.clone(),
                        saved_substitution: self.substitution.clone(),
                        saved_control_stack: self.control_stack.clone(),
                        alternative_clauses,
                        saved_uf: self.uf.clone(),
                    };
                    self.choice_stack.push(cp);
                    self.pc = jump_to;
                    return Ok(());
                } else {
                    return Err(MachineError::NoIndexedClause(predicate, key_vec[0].clone()));
                }
            } else {
                return Err(MachineError::NoIndexEntry(predicate, key_vec[0].clone()));
            }
        } else {
            Err(MachineError::PredicateNotInIndex(predicate))
        }
    }

    /// New: Executes a multi-indexed call using a list of registers as the index key.
    #[inline(always)]
    fn exec_multi_indexed_call(&mut self, predicate: String, index_registers: Vec<usize>) -> Result<(), MachineError> {
        let mut key_vec = Vec::new();
        for reg in index_registers {
            let term = self.registers.get(reg)
                .ok_or(MachineError::RegisterOutOfBounds(reg))?
                .clone()
                .ok_or(MachineError::UninitializedRegister(reg))?;
            key_vec.push(term);
        }
        if let Some(index_map) = self.index_table.get(&predicate) {
            if let Some(clauses) = index_map.get(&key_vec) {
                if !clauses.is_empty() {
                    let mut alternatives = clauses.clone();
                    let jump_to = alternatives.remove(0);
                    let alternative_clauses = if alternatives.is_empty() { None } else { Some(alternatives) };
                    let cp = ChoicePoint {
                        saved_pc: self.pc,
                        saved_registers: self.registers.clone(),
                        saved_substitution: self.substitution.clone(),
                        saved_control_stack: self.control_stack.clone(),
                        alternative_clauses,
                        saved_uf: self.uf.clone(),
                    };
                    self.choice_stack.push(cp);
                    self.pc = jump_to;
                    return Ok(());
                } else {
                    return Err(MachineError::NoIndexedClause(predicate, key_vec[0].clone()));
                }
            } else {
                return Err(MachineError::NoIndexEntry(predicate, key_vec[0].clone()));
            }
        } else {
            Err(MachineError::PredicateNotInIndex(predicate))
        }
    }

    #[inline(always)]
    fn exec_tail_call(&mut self, predicate: String) -> Result<(), MachineError> {
        let _ = self.environment_stack.pop();
        if let Some(builtin) = self.builtins.get(&predicate) {
            builtin(self)
        } else if let Some(clauses) = self.predicate_table.get(&predicate) {
            if !clauses.is_empty() {
                let mut alternatives = clauses.clone();
                let jump_to = alternatives.remove(0);
                let alternative_clauses = if alternatives.is_empty() { None } else { Some(alternatives) };
                let cp = ChoicePoint {
                    saved_pc: self.pc,
                    saved_registers: self.registers.clone(),
                    saved_substitution: self.substitution.clone(),
                    saved_control_stack: self.control_stack.clone(),
                    alternative_clauses,
                    saved_uf: self.uf.clone(),
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

    #[inline(always)]
    fn exec_assert_clause(&mut self, predicate: String, address: usize) -> Result<(), MachineError> {
        self.predicate_table
            .entry(predicate.clone())
            .or_insert_with(Vec::new)
            .push(address);
        // Optionally update the index table if desired.
        Ok(())
    }

    #[inline(always)]
    fn exec_retract_clause(&mut self, predicate: String, address: usize) -> Result<(), MachineError> {
        if let Some(clauses) = self.predicate_table.get_mut(&predicate) {
            if let Some(pos) = clauses.iter().position(|&a| a == address) {
                clauses.remove(pos);
                self.update_index_table_on_retract(&predicate, address);
                Ok(())
            } else {
                Err(MachineError::PredicateClauseNotFound(predicate))
            }
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    #[inline(always)]
    fn exec_cut(&mut self) -> Result<(), MachineError> {
        self.choice_stack.clear();
        Ok(())
    }

    #[inline(always)]
    fn exec_build_compound(&mut self, target: usize, functor: String, arg_registers: Vec<usize>) -> Result<(), MachineError> {
        let mut args = Vec::new();
        for &reg in &arg_registers {
            let term = self.registers.get(reg)
                .ok_or(MachineError::RegisterOutOfBounds(reg))?
                .clone()
                .ok_or(MachineError::UninitializedRegister(reg))?;
            args.push(term);
        }
        if let Some(slot) = self.registers.get_mut(target) {
            *slot = Some(Term::Compound(functor, args));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }

    /// Runs the machine until no more instructions are available.
    #[inline(always)]
    pub fn run(&mut self) -> Result<(), MachineError> {
        while self.pc < self.code.len() {
            self.step()?;
        }
        Ok(())
    }
}
