use crate::term::Term;

pub fn substitute(term: &Term, var: usize, replacement: &Term) -> Term {
    match term {
        Term::Var(v) => {
            if *v == var { replacement.clone() } else { term.clone() }
        },
        Term::Const(_) => term.clone(),
        Term::Compound(f, args) => {
            Term::Compound(f.clone(), args.iter().map(|t| substitute(t, var, replacement)).collect())
        },
        Term::Lambda(param, body) => {
            if *param == var {
                term.clone()
            } else {
                Term::Lambda(*param, Box::new(substitute(body, var, replacement)))
            }
        },
        Term::App(fun, arg) => {
            Term::App(Box::new(substitute(fun, var, replacement)), Box::new(substitute(arg, var, replacement)))
        },
    }
}

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
