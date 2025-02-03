use std::collections::HashMap;
use crate::term::Term;
use crate::machine::frame::Frame;
use crate::union_find::UnionFind;

// A choice point to support backtracking and clause selection.
// Saves the program counter, registers, substitution, the current length of the trail,
// and a list of alternative clause addresses for the current predicate call.
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: HashMap<usize, Term>,
    pub saved_trail_len: usize,
    pub saved_control_stack: Vec<Frame>,
    pub alternative_clauses: Option<Vec<usize>>,
    // NEW: Save a snapshot of the union–find state.
    pub saved_uf: UnionFind,
}
