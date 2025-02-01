use crate::term::Term;

// Recursively substitutes every free occurrence of variable `var` in `term`
// with the given `replacement`. (This is a simple version that does not handle
// variable capture. In a complete implementation, renaming of bound variables
// would be required.)
pub fn substitute(term: &Term, var: &str, replacement: &Term) -> Term {
    match term {
        Term::Var(v) => {
            if v == var {
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
            if param == var {
                term.clone()
            } else {
                Term::Lambda(param.clone(), Box::new(substitute(body, var, replacement)))
            }
        },
        Term::App(t1, t2) => {
            Term::App(
                Box::new(substitute(t1, var, replacement)),
                Box::new(substitute(t2, var, replacement))
            )
        },
    }
}

// Performs a single beta-reduction step on the given term if it is an application of a lambda abstraction.
// That is, it transforms (λ<param>.<body>) <arg> into <body>[<arg>/<param>].
pub fn beta_reduce(term: &Term) -> Term {
    match term {
        // Match an application where the function part is a lambda abstraction.
        Term::App(box Term::Lambda(param, body), arg) => {
            substitute(body, param, arg)
        },
        // Otherwise, no reduction is performed.
        _ => term.clone(),
    }
}