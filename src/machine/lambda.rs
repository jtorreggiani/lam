// src/machine/lambda.rs
//! Lambda calculus utilities: free variable collection, substitution, and beta reduction.

use std::collections::HashSet;
use crate::machine::term::Term;

/// Returns the set of free variable IDs in the term.
fn free_vars(term: &Term) -> HashSet<usize> {
    match term {
        Term::Var(v) => {
            let mut set = HashSet::new();
            set.insert(*v);
            set
        },
        Term::Const(_) | Term::Str(_) => HashSet::new(),
        Term::Compound(_, args) => {
            args.iter().fold(HashSet::new(), |mut acc, t| {
                acc.extend(free_vars(t));
                acc
            })
        },
        Term::Lambda(param, body) => {
            let mut vars = free_vars(body);
            vars.remove(param);
            vars
        },
        Term::App(fun, arg) => {
            let mut vars = free_vars(fun);
            vars.extend(free_vars(arg));
            vars
        },
        Term::Prob(inner) => free_vars(inner),
        Term::Constraint(_, terms) => {
            terms.iter().fold(HashSet::new(), |mut acc, t| {
                acc.extend(free_vars(t));
                acc
            })
        },
        Term::Modal(_, inner) => free_vars(inner),
        Term::Temporal(_, inner) => free_vars(inner),
        Term::HigherOrder(inner) => free_vars(inner),
    }
}

/// Generates a fresh variable ID not present in either term.
fn generate_fresh_var(term: &Term, replacement: &Term) -> usize {
    let union: HashSet<usize> = free_vars(term).union(&free_vars(replacement)).cloned().collect();
    let mut fresh = 0;
    while union.contains(&fresh) {
        fresh += 1;
    }
    fresh
}

/// Performs capture–avoiding substitution of variable `var` with `replacement` in `term`.
pub fn substitute(term: &Term, var: usize, replacement: &Term) -> Term {
    match term {
        Term::Var(v) => {
            if *v == var { replacement.clone() } else { term.clone() }
        },
        Term::Const(_) | Term::Str(_) => term.clone(),
        Term::Compound(f, args) => {
            Term::Compound(f.clone(), args.iter().map(|t| substitute(t, var, replacement)).collect())
        },
        Term::Lambda(param, body) => {
            if *param == var {
                term.clone()
            } else {
                let replacement_free = free_vars(replacement);
                if replacement_free.contains(param) {
                    let fresh = generate_fresh_var(term, replacement);
                    let renamed_body = substitute(body, *param, &Term::Var(fresh));
                    Term::Lambda(fresh, Box::new(substitute(&renamed_body, var, replacement)))
                } else {
                    Term::Lambda(*param, Box::new(substitute(body, var, replacement)))
                }
            }
        },
        Term::App(fun, arg) => {
            Term::App(Box::new(substitute(fun, var, replacement)), Box::new(substitute(arg, var, replacement)))
        },
        Term::Prob(inner) => Term::Prob(Box::new(substitute(inner, var, replacement))),
        Term::Constraint(name, terms) => {
            Term::Constraint(name.clone(), terms.iter().map(|t| substitute(t, var, replacement)).collect())
        },
        Term::Modal(op, inner) => Term::Modal(op.clone(), Box::new(substitute(inner, var, replacement))),
        Term::Temporal(op, inner) => Term::Temporal(op.clone(), Box::new(substitute(inner, var, replacement))),
        Term::HigherOrder(inner) => Term::HigherOrder(Box::new(substitute(inner, var, replacement))),
    }
}

/// Performs one step of beta reduction.
pub fn beta_reduce(term: &Term) -> Term {
    match term {
        Term::App(fun, arg) => {
            if let Term::Lambda(param, body) = &**fun {
                substitute(body, *param, arg)
            } else {
                Term::App(Box::new(beta_reduce(fun)), Box::new(beta_reduce(arg)))
            }
        },
        Term::Lambda(param, body) => Term::Lambda(*param, Box::new(beta_reduce(body))),
        Term::Compound(f, args) => Term::Compound(f.clone(), args.iter().map(beta_reduce).collect()),
        Term::Str(_) => term.clone(),
        Term::Prob(inner) => Term::Prob(Box::new(beta_reduce(inner))),
        Term::Constraint(name, terms) => Term::Constraint(name.clone(), terms.iter().map(beta_reduce).collect()),
        Term::Modal(op, inner) => Term::Modal(op.clone(), Box::new(beta_reduce(inner))),
        Term::Temporal(op, inner) => Term::Temporal(op.clone(), Box::new(beta_reduce(inner))),
        Term::HigherOrder(inner) => Term::HigherOrder(Box::new(beta_reduce(inner))),
        _ => term.clone(),
    }
}
