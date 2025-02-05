/// Represents the various kinds of terms in the LAM.
///
/// This definition has been extended to support higher-order logic, probabilistic programming,
/// constraints, modal logic, and temporal logic. These additional variants provide a foundation
/// for a rich language capable of expressing advanced logical constructs.
///
/// # Variants
/// - `Const(i32)`: Represents an integer constant.
/// - `Var(usize)`: Represents a variable with a unique identifier.
/// - `Compound(String, Vec<Term>)`: Represents a compound term (e.g., a structure or predicate).
/// - `Lambda(usize, Box<Term>)`: Represents a lambda abstraction, supporting higher-order functions.
/// - `App(Box<Term>, Box<Term>)`: Represents the application of one term to another.
/// - `Prob(Box<Term>)`: Represents a probabilistic term. This can encapsulate a distribution or a
///    random variable. For example, `Prob(Box::new( ... ))` may represent a term that yields
///    a value according to some probability distribution.
/// - `Constraint(String, Vec<Term>)`: Represents a constraint. The string field can identify the
///    type of constraint (e.g., equality, inequality, domain membership), and the vector holds the
///    involved terms.
/// - `Modal(String, Box<Term>)`: Represents a modal operator applied to a term. The string could be
///    something like "necessarily" or "possibly", and the inner term is the proposition.
/// - `Temporal(String, Box<Term>)`: Represents a temporal operator applied to a term. For example,
///    "always", "eventually", or "until" can be represented here.
/// - `HigherOrder(Box<Term>)`: Represents a higher-order term. Although `Lambda` and `App` already
///    provide basic higher-order functionality, this variant can be used to encapsulate or mark terms
///    that require special higher-order treatment (e.g., higher-order unification).
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    Const(i32),
    Var(usize),
    Compound(String, Vec<Term>),
    Lambda(usize, Box<Term>),
    App(Box<Term>, Box<Term>),
    // New variants for advanced capabilities:
    /// A probabilistic term (e.g., a random variable or distribution).
    Prob(Box<Term>),
    /// A constraint term, where the first field identifies the constraint type and the second
    /// holds the related terms.
    Constraint(String, Vec<Term>),
    /// A modal logic term (e.g., "necessarily" or "possibly").
    Modal(String, Box<Term>),
    /// A temporal logic term (e.g., "always", "eventually").
    Temporal(String, Box<Term>),
    /// A wrapper for higher-order terms that might need special handling.
    HigherOrder(Box<Term>),
}
