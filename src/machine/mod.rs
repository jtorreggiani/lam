// src/machine/mod.rs
//! The machine module.
//!
//! This module re-exports all the submodules that make up the LAM machine.

pub mod arithmetic;
pub mod error_handling;
pub mod instruction;
pub mod core;
pub mod term;
pub mod unification;
pub mod frame;
pub mod choice_point;
pub mod execution;
pub mod lambda;
