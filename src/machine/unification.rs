// src/machine/unification.rs
//! Union–find based unification for LAM.

use std::collections::HashMap;
use crate::machine::term::Term;
use crate::machine::error_handling::MachineError;

#[derive(Debug, Clone)]
pub struct UnionFind {
    bindings: HashMap<usize, Term>,
}

impl UnionFind {
    /// Creates a new UnionFind structure.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
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

    /// Binds the variable `var` to `term` (after resolution).
    pub fn bind(&mut self, var: usize, term: &Term) -> Result<(), MachineError> {
        let resolved_term = self.resolve(term);
        if let Term::Var(v) = &resolved_term {
            if *v == var {
                return Ok(());
            }
        }
        self.bindings.insert(var, resolved_term);
        Ok(())
    }
}
