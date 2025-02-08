// src/machine/unification.rs
//! Union–find based unification for LAM with trailing for efficient backtracking.

use std::collections::HashMap;
use crate::machine::term::Term;
use crate::machine::error_handling::MachineError;

/// Represents a trail entry recording a variable’s old binding.
#[derive(Debug, Clone)]
pub struct TrailEntry {
    pub var: usize,
    pub old_binding: Option<Term>,
}

/// Union–find structure with a trailing mechanism.
#[derive(Debug, Clone)]
pub struct UnionFind {
    pub bindings: HashMap<usize, Term>,
    pub trail: Vec<TrailEntry>,
}

impl UnionFind {
    /// Creates a new UnionFind structure.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            trail: Vec::new(),
        }
    }

    /// Recursively resolves a term to its current binding.
    pub fn resolve(&self, term: &Term) -> Term {
        match term {
            Term::Var(v) => {
                if let Some(binding) = self.bindings.get(v) {
                    self.resolve(binding)
                } else {
                    term.clone()
                }
            },
            _ => term.clone(),
        }
    }

    /// Binds the variable `var` to `term` (after resolution), recording the old binding on the trail.
    pub fn bind(&mut self, var: usize, term: &Term) -> Result<(), MachineError> {
        let resolved_term = self.resolve(term);
        // Avoid binding a variable to itself.
        if let Term::Var(v) = &resolved_term {
            if *v == var {
                return Ok(());
            }
        }
        // Record the current binding (if any) on the trail.
        let old_binding = self.bindings.get(&var).cloned();
        self.trail.push(TrailEntry { var, old_binding });
        self.bindings.insert(var, resolved_term);
        Ok(())
    }

    /// Rolls back the trail so that its length becomes `target_len`.
    pub fn undo_trail(&mut self, target_len: usize) {
        while self.trail.len() > target_len {
            if let Some(entry) = self.trail.pop() {
                if let Some(old) = entry.old_binding {
                    self.bindings.insert(entry.var, old);
                } else {
                    self.bindings.remove(&entry.var);
                }
            }
        }
    }
}
