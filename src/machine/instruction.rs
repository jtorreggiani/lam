use crate::term::Term;
use crate::arithmetic::Expression;
use crate::error_handling::MachineError;
use crate::core::Machine;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    // Puts a constant in a register.
    PutConst { register: usize, value: i32 },
    // Puts a variable in a register.
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
    Deallocate,
    SetLocal { index: usize, value: Term },
    GetLocal { index: usize, register: usize },
    // TailCall — a tail-recursive call to a predicate.
    TailCall { predicate: String },
    // IndexedCall uses a single register’s content as an index key.
    IndexedCall { predicate: String, index_register: usize },
    // MultiIndexedCall uses multiple registers to build an index key.
    MultiIndexedCall { predicate: String, index_registers: Vec<usize> },
    // Arithmetic instructions.
    ArithmeticIs { target: usize, expression: Expression },
    // AssertClause adds a clause address for a predicate.
    AssertClause { predicate: String, address: usize },
    // RetractClause removes a clause address for a predicate.
    RetractClause { predicate: String, address: usize },
    // Cut — prunes all choice points for the current predicate call.
    Cut,
    // Puts a string constant in a register.
    PutStr { register: usize, value: String },
    /// Unifies the term in the register with the given string constant.
    GetStr { register: usize, value: String },
    /// Halt — stops the machine execution.
    Halt,
}

impl Instruction {
    /// Executes the instruction on the provided machine.
    /// This method encapsulates the command logic for each instruction.
    pub fn execute(&self, machine: &mut Machine) -> Result<(), MachineError> {
        match self {
            Instruction::PutConst { register, value } => machine.execute_put_const(*register, *value),
            Instruction::PutVar { register, var_id, name } => machine.execute_put_var(*register, *var_id, name.clone()),
            Instruction::GetConst { register, value } => machine.execute_get_const(*register, *value),
            Instruction::GetVar { register, var_id, name } => machine.execute_get_var(*register, *var_id, name.clone()),
            Instruction::Call { predicate } => machine.execute_call(predicate.clone()),
            Instruction::Proceed => machine.execute_proceed(),
            Instruction::Choice { alternative } => machine.execute_choice(*alternative),
            Instruction::Allocate { n } => machine.execute_allocate(*n),
            Instruction::Deallocate => machine.execute_deallocate(),
            Instruction::ArithmeticIs { target, expression } => machine.execute_arithmetic_is(*target, expression.clone()),
            Instruction::SetLocal { index, value } => machine.execute_set_local(*index, value.clone()),
            Instruction::GetLocal { index, register } => machine.execute_get_local(*index, *register),
            Instruction::Fail => machine.execute_fail(),
            Instruction::GetStructure { register, functor, arity } => machine.execute_get_structure(*register, functor.clone(), *arity),
            Instruction::IndexedCall { predicate, index_register } => machine.execute_indexed_call(predicate.clone(), *index_register),
            Instruction::MultiIndexedCall { predicate, index_registers } => machine.execute_multi_indexed_call(predicate.clone(), index_registers.clone()),
            Instruction::TailCall { predicate } => machine.execute_tail_call(predicate.clone()),
            Instruction::AssertClause { predicate, address } => machine.execute_assert_clause(predicate.clone(), *address),
            Instruction::RetractClause { predicate, address } => machine.execute_retract_clause(predicate.clone(), *address),
            Instruction::Cut => machine.execute_cut(),
            Instruction::BuildCompound { target, functor, arg_registers } => machine.execute_build_compound(*target, functor.clone(), arg_registers.clone()),
            Instruction::PutStr { register, value } => machine.execute_put_str(*register, value.clone()),
            Instruction::GetStr { register, value } => machine.execute_get_str(*register, value.clone()),
            Instruction::Halt => Ok(()), // Halt will be handled in the run loop.
        }
    }
}
