use std::collections::HashMap;
use crate::term::Term;
use crate::machine::MachineError;

#[derive(Debug, Clone)]
pub struct UnionFind {
    // Mapping from variable id to the term it is bound to.
    pub parent: HashMap<usize, Term>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }

    /// Recursively resolves a term. If it is a variable and has a binding,
    /// follow the chain and apply path compression.
    pub fn resolve(&mut self, term: &Term) -> Term {
        match term {
            Term::Var(var_id) => {
                if let Some(binding) = self.parent.get(var_id) {
                    // Clone the binding so the immutable borrow ends.
                    let binding = binding.clone();
                    let resolved = self.resolve(&binding);
                    // Path compression: update the binding to the final resolved term.
                    self.parent.insert(*var_id, resolved.clone());
                    resolved
                } else {
                    term.clone()
                }
            },
            // For compounds, resolve all subterms.
            Term::Compound(f, args) => {
                Term::Compound(f.clone(), args.iter().map(|t| self.resolve(t)).collect())
            },
            // Lambdas and Applications can be left as is.
            Term::Lambda(param, body) => {
                Term::Lambda(*param, Box::new(self.resolve(body)))
            },
            Term::App(fun, arg) => {
                Term::App(Box::new(self.resolve(fun)), Box::new(self.resolve(arg)))
            },
            _ => term.clone(),
        }
    }

    /// Checks whether variable `var` occurs in `term`.
    fn occurs_check(&mut self, var: usize, term: &Term) -> bool {
        match self.resolve(term) {
            Term::Var(v) => v == var,
            Term::Compound(_, ref args) => args.iter().any(|t| self.occurs_check(var, t)),
            Term::Lambda(param, ref body) => {
                if param == var {
                    false
                } else {
                    self.occurs_check(var, body)
                }
            },
            Term::App(ref fun, ref arg) => self.occurs_check(var, fun) || self.occurs_check(var, arg),
            _ => false,
        }
    }

    /// Attempts to bind variable `var` to `term`. Performs an occurs check to avoid
    /// binding a variable to a term that contains it.
    pub fn bind(&mut self, var: usize, term: &Term) -> Result<(), MachineError> {
        if self.occurs_check(var, term) {
            Err(MachineError::UnificationFailed(format!(
                "Occurs check failed: variable {} occurs in {:?}",
                var, term
            )))
        } else {
            // Save the binding.
            self.parent.insert(var, term.clone());
            Ok(())
        }
    }
}
