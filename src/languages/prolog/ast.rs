// src/languages/prolog/ast.rs

/// The abstract syntax for Prolog terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrologTerm {
    /// A numeric constant.
    Const(i32),
    /// A string literal.
    Str(String),
    /// A variable (by name).
    Var(String),
    /// An atom.
    Atom(String),
    /// A compound term with a functor and a list of arguments.
    Compound(String, Vec<PrologTerm>),
}

/// An atomic goal: a predicate name and a (possibly empty) list of arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrologGoal {
    pub predicate: String,
    pub args: Vec<PrologTerm>,
}

/// A clause is either a fact or a rule (with an optional body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrologClause {
    pub head: PrologGoal,
    pub body: Option<Vec<PrologGoal>>,
}
