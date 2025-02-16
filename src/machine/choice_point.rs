// src/machine/choice_point.rs
//! Choice point structure for backtracking in the LAM.
//!
//! A choice point saves the state of the machine so that backtracking can restore it.

use std::collections::HashMap;
use crate::term::Term;
use crate::machine::frame::Frame;

/// A saved machine state used for backtracking.
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    /// Saved program counter.
    pub saved_pc: usize,
    /// Saved registers.
    pub saved_registers: Vec<Option<Term>>,
    /// Saved substitution mapping.
    pub saved_substitution: HashMap<usize, Term>,
    /// Saved control stack.
    pub saved_control_stack: Vec<Frame>,
    /// Alternative clause addresses for backtracking.
    pub alternative_clauses: Option<Vec<usize>>,
    /// The union–find trail length at the time of choice point creation.
    pub uf_trail_len: usize,
    /// The call level (e.g. the length of the control stack) when this choice point was created.
    pub call_level: usize,
}
