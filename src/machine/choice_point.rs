use crate::term::Term;
use crate::machine::frame::Frame;
use crate::union_find::UnionFind;

#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: std::collections::HashMap<usize, Term>,
    pub saved_trail_len: usize,
    pub saved_control_stack: Vec<Frame>,
    pub alternative_clauses: Option<Vec<usize>>,
    pub saved_uf: UnionFind,
}
