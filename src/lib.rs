//! LAM (Logical Abstract Machine) Library
//!
//! This library implements an abstract machine for logical inference,
//! combining features from Prolog, lambda calculus, and advanced logic.
//! The system is designed using several design patterns (Command, Strategy, Memento)
//! for clarity, debuggability, and extendability.

pub mod term;
pub mod machine;
pub mod lambda;
pub mod arithmetic;
pub mod union_find;
pub mod parser;
pub mod assembler;
