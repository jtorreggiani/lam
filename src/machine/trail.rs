use crate::term::Term;

#[derive(Debug, Clone)]
pub struct TrailEntry {
    pub variable: String,
    pub previous_value: Option<Term>,
}