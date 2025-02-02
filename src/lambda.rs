use crate::term::Term;

/// Recursively substitutes every free occurrence of variable `var` (identified by its unique id)
/// in `term` with the given `replacement`. (This is a simple version that does not handle
/// variable capture. In a complete implementation, renaming of bound variables would be required.)
pub fn substitute(term: &Term, var: usize, replacement: &Term) -> Term {
    match term {
        Term::Var(v) => {
            if *v == var {
                replacement.clone()
            } else {
                term.clone()
            }
        },
        Term::Const(_) => term.clone(),
        Term::Compound(f, args) => {
            Term::Compound(f.clone(), args.iter().map(|t| substitute(t, var, replacement)).collect())
        },
        Term::Lambda(param, body) => {
            // If the bound variable is the same as the one we're substituting, leave it unchanged.
            if *param == var {
                term.clone()
            } else {
                Term::Lambda(*param, Box::new(substitute(body, var, replacement)))
            }
        },
        Term::App(fun, arg) => {
            Term::App(
                Box::new(substitute(fun, var, replacement)),
                Box::new(substitute(arg, var, replacement))
            )
        },
    }
}

/// Performs a single beta-reduction step on the given term if it is an application of a lambda abstraction.
/// That is, it transforms (λ<var>.<body>) <arg> into <body>[<arg>/<var>].
pub fn beta_reduce(term: &Term) -> Term {
    match term {
        Term::App(fun, arg) => {
            if let Term::Lambda(param, body) = *fun.clone() {
                substitute(&body, param, arg)
            } else {
                term.clone()
            }
        },
        _ => term.clone(),
    }
}
