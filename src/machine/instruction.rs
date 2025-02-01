use crate::term::Term;

/// The set of instructions for our abstract machine.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    // Puts a constant in a register.
    PutConst { register: usize, value: i32 },
    // Puts a variable in a register.
    PutVar { register: usize, name: String },
    // Unifies the term in the register with the given constant.
    GetConst { register: usize, value: i32 },
    // Unifies the term in the register with a variable.
    GetVar { register: usize, name: String },
    // Calls a predicate by name.
    Call { predicate: String },
    // Proceeds (returns) from the current predicate.
    Proceed,
    // Creates a choice point (for backtracking).
    Choice,
    // Fails and triggers backtracking.
    Fail,
    // Constructs a compound term from registers.
    BuildCompound { target: usize, functor: String, arg_registers: Vec<usize> },
    // Checks that the term in the specified register is a compound term with the given functor and arity.
    GetStructure { register: usize, functor: String, arity: usize },
    // Environment management instructions.
    Allocate { n: usize },
    Deallocate,
    SetLocal { index: usize, value: Term },
    GetLocal { index: usize, register: usize },
    // TailCall — a tail-recursive call to a predicate.
    TailCall { predicate: String },
}