// src/machine/mod.rs
//! The machine module.
//!
//! This module re-exports all the submodules that make up the LAM machine.

pub mod arithmetic;
pub mod choice_point;
pub mod core;
pub mod error_handling;
pub mod execution;
pub mod frame;
pub mod instruction;
pub mod lambda;
pub mod term;
pub mod unification;
