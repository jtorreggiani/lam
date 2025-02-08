// src/languages/prolog/ast.rs

/// Represents a Prolog term.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// A variable is represented by its name.
    Var(String),
    /// An atom (a constant, typically starting with a lowercase letter).
    Atom(String),
    /// A number (we support integers for now).
    Number(i32),
    /// A compound term: functor with arguments.
    Compound(String, Vec<Term>),
}

/// Represents a Prolog clause.
#[derive(Debug, Clone, PartialEq)]
pub enum Clause {
    /// A fact is just a head term.
    Fact {
        head: Term,
    },
    /// A rule has a head and a body (a list of goals).
    Rule {
        head: Term,
        body: Vec<Term>,
    },
}

/// Represents a Prolog query, which is simply a sequence of goals.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub goals: Vec<Term>,
}
