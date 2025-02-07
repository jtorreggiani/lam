use std::fmt;

/// The different kinds of terms in our Prolog system.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    Const(i32),
    Var(usize),
    Compound(String, Vec<Term>),
    Lambda(usize, Box<Term>),
    App(Box<Term>, Box<Term>),
    Prob(Box<Term>),
    Constraint(String, Vec<Term>),
    Modal(String, Box<Term>),
    Temporal(String, Box<Term>),
    HigherOrder(Box<Term>),
    Str(String),
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Const(n) => write!(f, "{}", n),
            Term::Str(s) => write!(f, "{}", s),
            Term::Var(id) => write!(f, "Var({})", id),
            Term::Compound(functor, args) => {
                write!(f, "{}(", functor)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            // For the other variants, we fallback to the debug representation.
            other => write!(f, "{:?}", other),
        }
    }
}
