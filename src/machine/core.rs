//! Implements the LAM abstract machine.
//! This module uses various design patterns to structure the machine's behavior.
//! Detailed logging is included to trace every logical inference step for debugging purposes.

use std::collections::HashMap;
use log::{debug, error, info};

use crate::term::Term;
use crate::union_find::UnionFind;
use crate::error_handling::MachineError;

use crate::arithmetic;
use super::{
    instruction::Instruction,
    frame::Frame,
    choice_point::ChoicePoint,
};

/// Built-in predicate type using the **Strategy Pattern**.
pub type BuiltinPredicate = fn(&mut Machine) -> Result<(), MachineError>;

/// The LAM abstract machine structure.
#[derive(Debug)]
pub struct Machine {
    pub registers: Vec<Option<Term>>,
    pub code: Vec<Instruction>,
    pub pc: usize,
    pub substitution: HashMap<usize, Term>,
    pub control_stack: Vec<Frame>,
    pub predicate_table: HashMap<String, Vec<usize>>,
    pub choice_stack: Vec<ChoicePoint>,
    pub environment_stack: Vec<Vec<Option<Term>>>,
    pub index_table: HashMap<String, HashMap<Vec<Term>, Vec<usize>>>,
    pub variable_names: HashMap<usize, String>,
    pub uf: UnionFind,
    /// If true, print detailed execution trace.
    pub verbose: bool,
    /// Built-in predicates.
    pub builtins: HashMap<String, BuiltinPredicate>,
}

impl Machine {
    /// Creates a new machine with the specified number of registers and program code.
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
            uf: UnionFind::new(),
            variable_names: HashMap::new(),
            verbose: false,
            builtins: HashMap::new(),
        };
        // Register example built-in predicates.
        machine.builtins.insert("print".to_string(), Machine::builtin_print);
        machine.builtins.insert("print_subst".to_string(), Machine::builtin_print_subst);
        machine.builtins.insert("write".to_string(), Machine::builtin_write);
        machine.builtins.insert("nl".to_string(), Machine::builtin_nl);
        machine
    }

    /// Built-in predicate that prints the machine registers.
    fn builtin_print(&mut self) -> Result<(), MachineError> {
        println!("--- Machine Registers ---");
        for (i, reg) in self.registers.iter().enumerate() {
            if let Some(term) = reg {
                match term {
                    Term::Var(id) => {
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

    /// Built-in predicate that prints the current substitution.
    fn builtin_print_subst(&mut self) -> Result<(), MachineError> {
        println!("--- Current Substitution ---");
        if self.substitution.is_empty() {
            println!("(no bindings)");
        } else {
            for (var_id, term) in &self.substitution {
                let var_name = self.variable_names.get(var_id).cloned().unwrap_or_default();
                println!("Variable {} (id {}) = {:?}", if var_name.is_empty() { format!("_{}", var_id) } else { var_name }, var_id, term);
            }
        }
        println!("----------------------------");
        Ok(())
    }

    fn builtin_write(&mut self) -> Result<(), MachineError> {
        if let Some(Some(term)) = self.registers.get(0) {
            print!("{}", term);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    fn builtin_nl(&mut self) -> Result<(), MachineError> {
        println!();
        Ok(())
    }

    /// Logs the execution of an instruction if verbose mode is enabled.
    pub fn trace(&self, instr: &Instruction) {
        if self.verbose {
            debug!("PC {}: Executing {:?}", self.pc - 1, instr);
            debug!("Registers: {:?}", self.registers);
            debug!("Substitution: {:?}", self.substitution);
        }
    }

    /// Helper to update the index table upon clause retraction.
    fn update_index_table_on_retract(&mut self, predicate: &str, clause_address: usize) {
        if let Some(index_map) = self.index_table.get_mut(predicate) {
            for (_key, clauses) in index_map.iter_mut() {
                if let Some(pos) = clauses.iter().position(|&a| a == clause_address) {
                    clauses.remove(pos);
                }
            }
        }
    }

    /// Registers an indexed clause for the given predicate.
    pub fn register_indexed_clause(&mut self, predicate: String, key: Vec<Term>, address: usize) {
        let entry = self.index_table.entry(predicate).or_insert_with(HashMap::new);
        entry.entry(key).or_insert_with(Vec::new).push(address);
    }

    /// Registers a clause for the given predicate name.
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table.entry(name).or_insert_with(Vec::new).push(address);
    }

    /// Unifies two terms, logging each inference step.
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> Result<(), MachineError> {
        debug!("Attempting to unify {:?} with {:?}", t1, t2);
        let resolved1 = self.uf.resolve(t1);
        let resolved2 = self.uf.resolve(t2);
    
        match (&resolved1, &resolved2) {
            (Term::Const(a), Term::Const(b)) => {
                if a == b { 
                    debug!("Constants matched: {} == {}", a, b);
                    Ok(()) 
                } else {
                    Err(MachineError::UnificationFailed(format!("Constants do not match: {} vs {}", a, b)))
                }
            },
            (Term::Str(s1), Term::Str(s2)) => {
                if s1 == s2 { 
                    debug!("String constants matched: {} == {}", s1, s2);
                    Ok(()) 
                } else {
                    Err(MachineError::UnificationFailed(format!("String constants do not match: {} vs {}", s1, s2)))
                }
            },
            (Term::Var(v), other) => {
                debug!("Binding variable {} to {:?}", v, other);
                self.uf.bind(*v, other)
            },
            (other, Term::Var(v)) => {
                debug!("Binding variable {} to {:?}", v, other);
                self.uf.bind(*v, other)
            },
            (Term::Compound(f1, args1), Term::Compound(f2, args2)) => {
                if f1 != f2 || args1.len() != args2.len() {
                    return Err(MachineError::UnificationFailed(format!("Compound term mismatch: {} vs {}", f1, f2)));
                }
                for (a, b) in args1.iter().zip(args2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            },
            (t1, t2) => Err(MachineError::UnificationFailed(format!("Failed to unify {:?} with {:?}", t1, t2))),
        }
    }

    /// Executes one instruction from the code.
    pub fn step(&mut self) -> Result<(), MachineError> {
        let instr = self.code.get(self.pc)
            .ok_or(MachineError::NoMoreInstructions)?
            .clone();
        self.pc += 1;
        self.trace(&instr);
        // Use the Command Pattern: delegate execution to the instruction.
        instr.execute(self)
    }

    // --- Execution methods for each instruction ---

    pub fn execute_put_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Const(value));
            debug!("PutConst: Register {} set to Const({})", register, value);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    pub fn execute_put_var(&mut self, register: usize, var_id: usize, name: String) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Var(var_id));
            self.variable_names.insert(var_id, name);
            debug!("PutVar: Register {} set to Var({})", register, var_id);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    pub fn execute_get_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        match self.registers.get(register) {
            Some(Some(term)) => {
                let term_clone = term.clone();
                self.unify(&term_clone, &Term::Const(value))
                    .map_err(|_| MachineError::UnificationFailed(format!("Cannot unify {:?} with Const({})", term_clone, value)))
            },
            Some(None) => Err(MachineError::UninitializedRegister(register)),
            None => Err(MachineError::RegisterOutOfBounds(register)),
        }
    }

    pub fn execute_get_var(&mut self, register: usize, var_id: usize, name: String) -> Result<(), MachineError> {
        if register >= self.registers.len() {
            return Err(MachineError::RegisterOutOfBounds(register));
        }
        self.variable_names.entry(var_id).or_insert(name);
        if let Some(term) = self.registers[register].clone() {
            let goal = Term::Var(var_id);
            self.unify(&goal, &term)
                .map_err(|_| MachineError::UnificationFailed(format!("Cannot unify {:?} with {:?}", goal, term)))
        } else {
            self.registers[register] = Some(Term::Var(var_id));
            Ok(())
        }
    }

    pub fn execute_call(&mut self, predicate: String) -> Result<(), MachineError> {
        debug!("Executing Call for predicate '{}'", predicate);
        if let Some(builtin) = self.builtins.get(&predicate) {
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
            debug!("Call: Jumping to clause at address {}", jump_to);
            self.pc = jump_to;
            Ok(())
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    pub fn execute_proceed(&mut self) -> Result<(), MachineError> {
        if let Some(frame) = self.control_stack.pop() {
            debug!("Proceed: Returning to PC {}", frame.return_pc);
            self.pc = frame.return_pc;
        }
        Ok(())
    }

    pub fn execute_choice(&mut self, alternative: usize) -> Result<(), MachineError> {
        let cp = ChoicePoint {
            saved_pc: self.pc,
            saved_registers: self.registers.clone(),
            saved_substitution: self.substitution.clone(),
            saved_control_stack: self.control_stack.clone(),
            alternative_clauses: Some(vec![alternative]),
            saved_uf: self.uf.clone(),
        };
        self.choice_stack.push(cp);
        debug!("Choice: Saved state with alternative {}", alternative);
        Ok(())
    }

    pub fn execute_allocate(&mut self, n: usize) -> Result<(), MachineError> {
        self.environment_stack.push(vec![None; n]);
        debug!("Allocate: Environment frame of size {} allocated", n);
        Ok(())
    }

    pub fn execute_deallocate(&mut self) -> Result<(), MachineError> {
        if self.environment_stack.pop().is_some() {
            debug!("Deallocate: Environment frame deallocated");
            Ok(())
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    pub fn execute_arithmetic_is(&mut self, target: usize, expression: arithmetic::Expression) -> Result<(), MachineError> {
        let result = arithmetic::evaluate(&expression, &self.registers)?;
        if let Some(slot) = self.registers.get_mut(target) {
            *slot = Some(Term::Const(result));
            debug!("ArithmeticIs: Evaluated expression to {} and stored in register {}", result, target);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }

    pub fn execute_set_local(&mut self, index: usize, value: Term) -> Result<(), MachineError> {
        if let Some(env) = self.environment_stack.last_mut() {
            if let Some(slot) = env.get_mut(index) {
                *slot = Some(value.clone());
                debug!("SetLocal: Environment variable at index {} set to {:?}", index, value);
                Ok(())
            } else {
                Err(MachineError::RegisterOutOfBounds(index))
            }
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    pub fn execute_get_local(&mut self, index: usize, register: usize) -> Result<(), MachineError> {
        if let Some(env) = self.environment_stack.last() {
            let term = env.get(index).and_then(|t| t.clone()).ok_or(MachineError::UninitializedRegister(index))?;
            if let Some(reg_slot) = self.registers.get_mut(register) {
                match reg_slot {
                    Some(existing_term) => {
                        let cloned = existing_term.clone();
                        self.unify(&cloned, &term)
                            .map_err(|_| MachineError::UnificationFailed(format!("Cannot unify {:?} with {:?}", cloned, term)))
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

    pub fn execute_fail(&mut self) -> Result<(), MachineError> {
        debug!("Fail: Triggering backtracking");
        while let Some(cp) = self.choice_stack.pop() {
            self.registers = cp.saved_registers;
            self.substitution = cp.saved_substitution;
            self.control_stack = cp.saved_control_stack;
            self.uf = cp.saved_uf;

            if let Some(mut alternatives) = cp.alternative_clauses {
                if let Some(next_addr) = alternatives.pop() {
                    if !alternatives.is_empty() {
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
                    debug!("Backtracking: Jumping to alternative clause at address {}", next_addr);
                    return Ok(());
                }
            }
        }
        Err(MachineError::NoChoicePoint)
    }

    pub fn execute_get_structure(&mut self, register: usize, functor: String, arity: usize) -> Result<(), MachineError> {
        match self.registers.get(register).and_then(|t| t.clone()) {
            Some(Term::Compound(ref f, ref args)) => {
                if f == &functor && args.len() == arity {
                    debug!("GetStructure: Register {} matches structure {} with arity {}", register, functor, arity);
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

    pub fn execute_indexed_call(&mut self, predicate: String, index_register: usize) -> Result<(), MachineError> {
        let key_term = self.registers.get(index_register)
            .ok_or(MachineError::RegisterOutOfBounds(index_register))?
            .clone()
            .ok_or(MachineError::UninitializedRegister(index_register))?;
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
                    debug!("IndexedCall: Jumping to clause at address {} using key {:?}", jump_to, key_vec);
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

    pub fn execute_put_str(&mut self, register: usize, value: String) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Str(value.clone()));
            debug!("PutStr: Register {} set to Str({})", register, value);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    pub fn execute_get_str(&mut self, register: usize, value: String) -> Result<(), MachineError> {
        match self.registers.get(register) {
            Some(Some(term)) => {
                let term_clone = term.clone();
                self.unify(&term_clone, &Term::Str(value.clone()))
                    .map_err(|_| MachineError::UnificationFailed(format!(
                        "Cannot unify {:?} with Str({})", term_clone, value
                    )))
            },
            Some(None) => Err(MachineError::UninitializedRegister(register)),
            None => Err(MachineError::RegisterOutOfBounds(register)),
        }
    }

    pub fn execute_multi_indexed_call(&mut self, predicate: String, index_registers: Vec<usize>) -> Result<(), MachineError> {
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
                    debug!("MultiIndexedCall: Jumping to clause at address {} using key {:?}", jump_to, key_vec);
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

    pub fn execute_tail_call(&mut self, predicate: String) -> Result<(), MachineError> {
        // Tail call deallocates the current environment frame.
        let _ = self.environment_stack.pop();
        debug!("TailCall: Deallocated environment frame for tail call to '{}'", predicate);
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
                debug!("TailCall: Jumping to clause at address {} for predicate '{}'", jump_to, predicate);
                Ok(())
            } else {
                Err(MachineError::PredicateClauseNotFound(predicate))
            }
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    pub fn execute_assert_clause(&mut self, predicate: String, address: usize) -> Result<(), MachineError> {
        self.predicate_table.entry(predicate.clone()).or_insert_with(Vec::new).push(address);
        debug!("AssertClause: Asserted clause at address {} for predicate '{}'", address, predicate);
        Ok(())
    }

    pub fn execute_retract_clause(&mut self, predicate: String, address: usize) -> Result<(), MachineError> {
        if let Some(clauses) = self.predicate_table.get_mut(&predicate) {
            if let Some(pos) = clauses.iter().position(|&a| a == address) {
                clauses.remove(pos);
                self.update_index_table_on_retract(&predicate, address);
                debug!("RetractClause: Retracted clause at address {} for predicate '{}'", address, predicate);
                Ok(())
            } else {
                Err(MachineError::PredicateClauseNotFound(predicate))
            }
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    pub fn execute_cut(&mut self) -> Result<(), MachineError> {
        self.choice_stack.clear();
        debug!("Cut: Cleared all choice points");
        Ok(())
    }

    pub fn execute_build_compound(&mut self, target: usize, functor: String, arg_registers: Vec<usize>) -> Result<(), MachineError> {
        let mut args = Vec::new();
        for &reg in &arg_registers {
            let term = self.registers.get(reg)
                .ok_or(MachineError::RegisterOutOfBounds(reg))?
                .clone()
                .ok_or(MachineError::UninitializedRegister(reg))?;
            args.push(term);
        }
        if let Some(slot) = self.registers.get_mut(target) {
            *slot = Some(Term::Compound(functor.clone(), args));
            debug!("BuildCompound: Built compound term {}({:?}) in register {}", functor, arg_registers, target);
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }

    /// Runs the machine until no more instructions are available.
    pub fn run(&mut self) -> Result<(), MachineError> {
        while self.pc < self.code.len() {
            // Check for Halt instruction explicitly.
            if let Some(Instruction::Halt) = self.code.get(self.pc) {
                debug!("Halt: Stopping execution");
                break;
            }
            self.step()?;
        }
        Ok(())
    }
}

pub fn ping() -> &'static str {
    "pong"
}
