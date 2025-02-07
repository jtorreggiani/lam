// src/machine/frame.rs
//! Frame structure used to store a return address in the machine’s control stack.

#[derive(Debug, Clone)]
pub struct Frame {
    /// The program counter to return to.
    pub return_pc: usize,
}
