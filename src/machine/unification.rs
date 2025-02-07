use std::collections::HashMap;
use crate::term::Term;
use crate::machine::MachineError;

#[derive(Debug, Clone)]
pub struct UnionFind {
    bindings: HashMap<usize, Term>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

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
