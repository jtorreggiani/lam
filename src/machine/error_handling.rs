// src/machine/error_handling.rs
//! Error handling for the LAM machine.
//!
//! This module defines the `MachineError` enum for reporting various errors.

use thiserror::Error;
use crate::term::Term;

#[derive(Debug, Error)]
pub enum MachineError {
    #[error("Register {0} is out of bounds.")]
    RegisterOutOfBounds(usize),
    #[error("Register {0} is uninitialized.")]
    UninitializedRegister(usize),
    #[error("Unification failed: {0}")]
    UnificationFailed(String),
    #[error("Environment missing.")]
    EnvironmentMissing,
    #[error("Predicate not found: {0}")]
    PredicateNotFound(String),
    #[error("Predicate clause not found: {0}")]
    PredicateClauseNotFound(String),
    #[error("No choice point available.")]
    NoChoicePoint,
    #[error("Structure mismatch: expected {expected_functor}/{expected_arity} but found {found_functor}/{found_arity}.")]
    StructureMismatch {
        expected_functor: String,
        expected_arity: usize,
        found_functor: String,
        found_arity: usize,
    },
    #[error("Term in register {0} is not a compound term.")]
    NotACompoundTerm(usize),
    #[error("No indexed clause for predicate {0} with key {1:?}.")]
    NoIndexedClause(String, Term),
    #[error("No index entry for predicate {0} with key {1:?}.")]
    NoIndexEntry(String, Term),
    #[error("Predicate {0} is not in the index.")]
    PredicateNotInIndex(String),
    #[error("No more instructions.")]
    NoMoreInstructions,
}
