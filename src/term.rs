//! Defines the various term types used in the LAM.
//! Terms represent data and logic constructs such as constants, variables, compounds, lambda abstractions, etc.

/// Represents the different kinds of terms in LAM.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    Const(i32),
    Var(usize),
    Compound(String, Vec<Term>),
    Lambda(usize, Box<Term>),
    App(Box<Term>, Box<Term>),
    /// Represents a probabilistic term.
    Prob(Box<Term>),
    /// Represents a constraint term.
    Constraint(String, Vec<Term>),
    /// Represents a modal logic term.
    Modal(String, Box<Term>),
    /// Represents a temporal logic term.
    Temporal(String, Box<Term>),
    /// Represents a higher-order term.
    HigherOrder(Box<Term>),
    Str(String),
}
