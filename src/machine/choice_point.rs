use std::collections::HashMap;
use crate::term::Term;
use crate::frame::Frame;
use crate::unification::UnionFind;

#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: HashMap<usize, Term>,
    pub saved_control_stack: Vec<Frame>,
    pub alternative_clauses: Option<Vec<usize>>,
    pub saved_uf: UnionFind,
}
