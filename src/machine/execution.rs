// src/machine/execution.rs
//! Execution methods for LAM instructions.
//!
//! This module implements the methods that execute each instruction on a Machine.

use crate::machine::core::Machine;
use crate::machine::frame::Frame;
use crate::machine::choice_point::ChoicePoint;
use crate::machine::term::Term;
use crate::machine::error_handling::MachineError;
use crate::machine::arithmetic;

impl Machine {
    pub fn execute_put_const(&mut self, register: usize, value: i32) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Const(value));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    pub fn execute_put_var(&mut self, register: usize, var_id: usize, name: String) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Var(var_id));
            self.variable_names.insert(var_id, name);
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
                .map_err(|_| MachineError::UnificationFailed(format!("Cannot unify {:?} with {:?}", goal, term)))?;
            let resolved = self.uf.resolve(&term);
            self.registers[register] = Some(resolved);
            Ok(())
        } else {
            self.registers[register] = Some(Term::Var(var_id));
            Ok(())
        }
    }

    pub fn execute_call(&mut self, predicate: String) -> Result<(), MachineError> {
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
                uf_trail_len: self.uf.trail.len(),
                call_level: self.control_stack.len(),
            };
            self.choice_stack.push(Box::new(cp));
            self.pc = jump_to;
            Ok(())
        } else {
            Err(MachineError::PredicateNotFound(predicate))
        }
    }

    pub fn execute_proceed(&mut self) -> Result<(), MachineError> {
        if let Some(frame) = self.control_stack.pop() {
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
            uf_trail_len: self.uf.trail.len(),
            call_level: self.control_stack.len(),
        };
        self.choice_stack.push(Box::new(cp));
        Ok(())
    }

    pub fn execute_allocate(&mut self, n: usize) -> Result<(), MachineError> {
        self.environment_stack.push(vec![None; n]);
        Ok(())
    }

    pub fn execute_deallocate(&mut self) -> Result<(), MachineError> {
        if self.environment_stack.pop().is_some() {
            Ok(())
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    pub fn execute_arithmetic_is(&mut self, target: usize, expression: arithmetic::Expression) -> Result<(), MachineError> {
        let result = arithmetic::evaluate(&expression, &self.registers)?;
        if let Some(slot) = self.registers.get_mut(target) {
            *slot = Some(Term::Const(result));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }

    pub fn execute_set_local(&mut self, index: usize, value: Term) -> Result<(), MachineError> {
        if let Some(env) = self.environment_stack.last_mut() {
            if let Some(slot) = env.get_mut(index) {
                *slot = Some(value.clone());
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
                if let Some(existing_term) = reg_slot {
                    let cloned = existing_term.clone();
                    self.unify(&cloned, &term)
                        .map_err(|_| MachineError::UnificationFailed(format!("Cannot unify {:?} with {:?}", cloned, term)))
                } else {
                    *reg_slot = Some(term);
                    Ok(())
                }
            } else {
                Err(MachineError::RegisterOutOfBounds(register))
            }
        } else {
            Err(MachineError::EnvironmentMissing)
        }
    }

    pub fn execute_fail(&mut self) -> Result<(), MachineError> {
        while let Some(cp_box) = self.choice_stack.pop() {
            let cp = *cp_box;
            self.registers = cp.saved_registers;
            self.substitution = cp.saved_substitution;
            self.control_stack = cp.saved_control_stack;
            // Roll back union–find bindings to the saved trail length.
            self.uf.undo_trail(cp.uf_trail_len);
            if let Some(mut alternatives) = cp.alternative_clauses {
                if let Some(next_addr) = alternatives.pop() {
                    if !alternatives.is_empty() {
                        let new_cp = ChoicePoint {
                            saved_pc: cp.saved_pc,
                            saved_registers: self.registers.clone(),
                            saved_substitution: self.substitution.clone(),
                            saved_control_stack: self.control_stack.clone(),
                            alternative_clauses: Some(alternatives),
                            uf_trail_len: self.uf.trail.len(),
                            call_level: cp.call_level,
                        };
                        self.choice_stack.push(Box::new(new_cp));
                    }
                    self.pc = next_addr;
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
                        uf_trail_len: self.uf.trail.len(),
                        call_level: self.control_stack.len(),
                    };
                    self.choice_stack.push(Box::new(cp));
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

    pub fn execute_put_str(&mut self, register: usize, value: String) -> Result<(), MachineError> {
        if let Some(slot) = self.registers.get_mut(register) {
            *slot = Some(Term::Str(value));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(register))
        }
    }

    pub fn execute_get_str(&mut self, register: usize, value: String) -> Result<(), MachineError> {
        let term_option = self.registers.get(register).cloned()
            .ok_or(MachineError::RegisterOutOfBounds(register))?;
        match term_option {
            Some(term) => {
                let term_clone = term.clone();
                self.unify(&term_clone, &Term::Str(value.clone()))
                    .map_err(|_| MachineError::UnificationFailed(format!("Cannot unify {:?} with Str({})", term_clone, value)))?;
                let resolved = self.uf.resolve(&term);
                self.registers[register] = Some(resolved);
                Ok(())
            },
            None => Err(MachineError::UninitializedRegister(register)),
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
                        uf_trail_len: self.uf.trail.len(),
                        call_level: self.control_stack.len(),
                    };
                    self.choice_stack.push(Box::new(cp));
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

    pub fn execute_tail_call(&mut self, predicate: String) -> Result<(), MachineError> {
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
                    uf_trail_len: self.uf.trail.len(),
                    call_level: self.control_stack.len(),
                };
                self.choice_stack.push(Box::new(cp));
                self.pc = jump_to;
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
        Ok(())
    }

    pub fn execute_retract_clause(&mut self, predicate: String, address: usize) -> Result<(), MachineError> {
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

    pub fn execute_cut(&mut self) -> Result<(), MachineError> {
        let current_call_level = self.control_stack.len();
        self.choice_stack.retain(|cp| cp.call_level < current_call_level);
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
            *slot = Some(Term::Compound(functor, args));
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds(target))
        }
    }
}
