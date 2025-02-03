#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    Const(i32),
    Var(usize),
    Compound(String, Vec<Term>),
    Lambda(usize, Box<Term>),
    App(Box<Term>, Box<Term>),
}
