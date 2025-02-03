use std::collections::HashMap;
use crate::term::Term;
use crate::machine::machine::MachineError;

#[derive(Debug, Clone)]
pub struct UnionFind {
    /// Maps variable id to its parent variable id.
    parent: HashMap<usize, usize>,
    /// Maps the representative variable id to its binding (if any).
    binding: HashMap<usize, Term>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            binding: HashMap::new(),
        }
    }

    /// Finds the representative for a variable, compressing paths along the way.
    pub fn find(&mut self, var: usize) -> usize {
        // Initialize self-parenting if necessary.
        if !self.parent.contains_key(&var) {
            self.parent.insert(var, var);
            return var;
        }
        let parent = self.parent[&var];
        if parent != var {
            let rep = self.find(parent);
            self.parent.insert(var, rep); // Path compression.
            rep
        } else {
            var
        }
    }

    /// Unifies two variables by merging their equivalence classes.
    pub fn union(&mut self, var1: usize, var2: usize) {
        let rep1 = self.find(var1);
        let rep2 = self.find(var2);
        if rep1 != rep2 {
            // For simplicity, always attach rep2 to rep1.
            self.parent.insert(rep2, rep1);
            // If one representative is bound and the other isn’t, propagate the binding.
            if let Some(term) = self.binding.get(&rep2).cloned() {
                self.binding.insert(rep1, term);
            }
        }
    }

    /// Iteratively checks whether `var` occurs in `term` to prevent cyclic bindings.
    /// This is our iterative occurs check.
    pub fn occurs_check(term: &Term, var: usize) -> bool {
        let mut stack = vec![term];
        while let Some(current) = stack.pop() {
            match current {
                crate::term::Term::Var(v) if *v == var => return true,
                crate::term::Term::Compound(_, args) => stack.extend(args.iter()),
                crate::term::Term::Lambda(param, body) => {
                    if *param != var {
                        stack.push(body);
                    }
                },
                crate::term::Term::App(fun, arg) => {
                    stack.push(fun);
                    stack.push(arg);
                },
                _ => {}
            }
        }
        false
    }

    /// Binds the variable `var` (its representative) to a term.
    /// Returns an error if the occurs check fails.
    pub fn bind(&mut self, var: usize, term: &Term) -> Result<(), MachineError> {
        if UnionFind::occurs_check(term, var) {
            return Err(MachineError::UnificationFailed(format!(
                "Occurs check failed: variable {} in term {:?}",
                var, term
            )));
        }
        let rep = self.find(var);
        self.binding.insert(rep, term.clone());
        Ok(())
    }

    /// Resolves a term: if it is a variable and is bound, return its binding recursively.
    pub fn resolve(&mut self, term: &Term) -> Term {
        match term {
            Term::Var(v) => {
                // First find the representative for `v` immutably.
                let rep = self.find(*v);
                if let Some(bound) = self.binding.get(&rep).cloned() {
                    let resolved_bound = self.resolve(&bound);
                    return resolved_bound;
                } else {
                    return term.clone();
                }
            },
            _ => term.clone(), // Other types are not affected.
        }
    }
}
