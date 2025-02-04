// File: src/machine/choice_point.rs

use crate::term::Term;
use crate::machine::frame::Frame;
use crate::union_find::UnionFind;

/// Represents a saved state for backtracking.
/// 
/// **Note:** The field `saved_trail_len` has been removed as the trail mechanism
/// has been deprecated in favor of the union-find rollback.
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: std::collections::HashMap<usize, Term>,
    // Field removed: saved_trail_len
    pub saved_control_stack: Vec<Frame>,
    pub alternative_clauses: Option<Vec<usize>>,
    pub saved_uf: UnionFind,
}
