/// Represents a term in our logic programming language.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    /// A constant value (for example, a 32‐bit integer).
    Const(i32),
    /// A variable, identified by a unique id.
    Var(usize),
    /// A compound term with a functor and a list of arguments.
    Compound(String, Vec<Term>),
    /// A lambda abstraction: λ<var_id>.<body>
    /// (The variable’s unique id is stored in the term; its string name is stored in the machine.)
    Lambda(usize, Box<Term>),
    /// An application: (<function> <argument>)
    App(Box<Term>, Box<Term>),
}
