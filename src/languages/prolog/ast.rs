/// The abstract syntax for Prolog terms.
/// We support numeric constants, strings, variables, and atoms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrologTerm {
    Const(i32),
    Str(String),
    Var(String),
    Atom(String),
}

/// An atomic goal: a predicate name with a (possibly empty) list of arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrologGoal {
    pub predicate: String,
    pub args: Vec<PrologTerm>,
}

/// A clause is a fact or a rule. In a rule the head is separated from the body by “:-”.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrologClause {
    pub head: PrologGoal,
    pub body: Option<Vec<PrologGoal>>,
}
