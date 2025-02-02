use crate::term::Term;

#[derive(Debug, Clone)]
pub struct TrailEntry {
    pub variable: usize,
    pub previous_value: Option<Term>,
}
