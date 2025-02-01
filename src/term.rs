/// Represents a term in our logic programming language.
/// For now, we'll keep it simple with only two variants.
#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    /// A constant value (for example, a 32-bit integer).
    Const(i32),
    /// A variable, identified by a name.
    Var(String),
}
