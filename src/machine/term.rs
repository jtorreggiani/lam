// src/machine/term.rs
//! Definition of terms in the LAM system.

use std::fmt;

/// The various types of terms.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    /// A constant integer.
    Const(i32),
    /// A variable (identified by an ID).
    Var(usize),
    /// A compound term with a functor and a list of arguments.
    Compound(String, Vec<Term>),
    /// A lambda abstraction (parameter and body).
    Lambda(usize, Box<Term>),
    /// Function application.
    App(Box<Term>, Box<Term>),
    /// A probabilistic term.
    Prob(Box<Term>),
    /// A constraint with a name and arguments.
    Constraint(String, Vec<Term>),
    /// A modal term.
    Modal(String, Box<Term>),
    /// A temporal term.
    Temporal(String, Box<Term>),
    /// A higher–order term.
    HigherOrder(Box<Term>),
    /// A string constant.
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
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            other => write!(f, "{:?}", other),
        }
    }
}
