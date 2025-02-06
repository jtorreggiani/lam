//! Defines the structure for a choice point in the LAM.
//! The choice point stores the machine's state to allow backtracking,
//! following the **Memento Pattern**.

use crate::term::Term;
use crate::machine::frame::Frame;
use crate::union_find::UnionFind;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: HashMap<usize, Term>,
    pub saved_control_stack: Vec<Frame>,
    pub alternative_clauses: Option<Vec<usize>>,
    pub saved_uf: UnionFind,
}
