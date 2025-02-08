// src/lib.rs
//! LAM (Logical Abstract Machine) library.
//!
//! This library implements an abstract machine for logic programming with support
//! for unification, backtracking, arithmetic evaluation, and lambda calculus.
//!
//! Modules:
//! - machine: Contains the core machine implementation and supporting components.

// Re-export the machine modules for use in the library.
pub mod machine;
pub mod prolog;

// Re-export the specific modules for use in the library.
pub use machine::core;
pub use machine::term;
pub use machine::error_handling;
pub use machine::arithmetic;
pub use machine::instruction;

