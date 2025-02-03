use crate::term::Term;
use crate::arithmetic::Expression;

// The set of instructions for our abstract machine.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    // Puts a constant in a register.
    PutConst { register: usize, value: i32 },
    // Puts a variable in a register. The variable is identified by a unique id and its string name.
    PutVar { register: usize, var_id: usize, name: String },
    // Unifies the term in the register with the given constant.
    GetConst { register: usize, value: i32 },
    // Unifies the term in the register with a variable.
    GetVar { register: usize, var_id: usize, name: String },
    // Calls a predicate by name.
    Call { predicate: String },
    // Proceeds (returns) from the current predicate.
    Proceed,
    // Creates a choice point (for backtracking).
    Choice { alternative: usize },
    // Fails and triggers backtracking.
    Fail,
    // Constructs a compound term from registers.
    BuildCompound { target: usize, functor: String, arg_registers: Vec<usize> },
    // Checks that the term in the specified register is a compound term with the given functor and arity.
    GetStructure { register: usize, functor: String, arity: usize },
    // Environment management instructions.
    Allocate { n: usize },
    // Deallocates the top n frames from the environment stack.
    Deallocate,
    // Sets a local variable in the current environment frame.
    SetLocal { index: usize, value: Term },
    // Retrieves a local variable from the current environment frame.
    GetLocal { index: usize, register: usize },
    // TailCall — a tail-recursive call to a predicate.
    TailCall { predicate: String },
    // IndexedCall uses the content of the specified register as an index key.
    IndexedCall { predicate: String, index_register: usize },
    // Arithmetic instructions.
    ArithmeticIs { target: usize, expression: Expression },
    // AssertClause adds a clause address for a predicate.
    AssertClause { predicate: String, address: usize },
    // RetractClause removes a clause address for a predicate.
    RetractClause { predicate: String, address: usize },
    // Cut — prunes all choice points for the current predicate call.
    Cut,
}
