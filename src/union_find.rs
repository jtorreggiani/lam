// File: src/union_find.rs

use std::collections::HashMap;
use crate::term::Term;
use crate::machine::MachineError;

/// A simple union-find structure to manage variable bindings.
///
/// This implementation supports recursive resolution and binding of variables.
/// For backtracking, the entire union-find state is cloned, so incremental rollback is not required.
///
/// # Examples
/// ```
/// use lam::union_find::UnionFind;
/// use lam::term::Term;
/// use lam::machine::MachineError;
///
/// let mut uf = UnionFind::new();
/// // Initially, variable 0 is unbound.
/// assert_eq!(uf.resolve(&Term::Var(0)), Term::Var(0));
/// 
/// // Bind variable 0 to constant 42.
/// uf.bind(0, &Term::Const(42)).unwrap();
/// assert_eq!(uf.resolve(&Term::Var(0)), Term::Const(42));
/// ```
#[derive(Debug, Clone)]
pub struct UnionFind {
    /// Maps variable IDs to their binding term.
    bindings: HashMap<usize, Term>,
}

impl UnionFind {
    /// Creates a new, empty union-find.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Recursively resolves a term. If the term is a variable and is bound,
    /// returns its final binding. Otherwise, returns the term itself.
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

    /// Binds a variable to a term. The term is first resolved recursively.
    ///
    /// # Parameters
    /// - `var`: The variable ID to bind.
    /// - `term`: The term to bind to.
    ///
    /// # Returns
    /// - `Ok(())` if the binding succeeds.
    /// - `Err(MachineError::UnificationFailed(...))` if the binding is inconsistent (currently not implemented).
    pub fn bind(&mut self, var: usize, term: &Term) -> Result<(), MachineError> {
        let resolved_term = self.resolve(term);
        // Prevent self-binding loops.
        if let Term::Var(v) = &resolved_term {
            if *v == var {
                return Ok(())
            }
        }
        self.bindings.insert(var, resolved_term);
        Ok(())
    }
}
