// src/machine/unification.rs
//! Union–find based unification for LAM with trailing for efficient backtracking.
//!
//! This version implements path compression to reduce the cost of repeated resolution.
//! When a variable’s binding is resolved recursively, we update it (path compression)
//! and record the previous binding on the trail so that backtracking can restore the state.

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

    /// Recursively resolves a term to its current binding using path compression.
    ///
    /// If the term is a variable and has a binding, the method recursively resolves that binding.
    /// Then, if the current binding is not the fully resolved term, the binding is updated (path compression)
    /// and the previous binding is recorded on the trail so that backtracking can restore it.
    ///
    /// **Note:** This method now requires a mutable reference to self.
    pub fn resolve(&mut self, term: &Term) -> Term {
        match term {
            Term::Var(v) => {
                // Clone the binding out of the HashMap to release the immutable borrow.
                if let Some(binding) = self.bindings.get(v).cloned() {
                    // Recursively resolve the binding.
                    let resolved = self.resolve(&binding);
                    // If the binding is not yet compressed, update it and record the old value.
                    if binding != resolved {
                        self.trail.push(TrailEntry { var: *v, old_binding: Some(binding) });
                        self.bindings.insert(*v, resolved.clone());
                    }
                    resolved
                } else {
                    term.clone()
                }
            },
            _ => term.clone(),
        }
    }

    /// Binds the variable `var` to `term` (after resolution), recording the previous binding on the trail.
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
