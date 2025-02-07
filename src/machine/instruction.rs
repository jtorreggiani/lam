// src/machine/instruction.rs
//! Definitions of LAM instructions.

use crate::machine::term::Term;
use crate::machine::arithmetic::Expression;
use crate::machine::error_handling::MachineError;
use crate::machine::core::Machine;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    PutConst { register: usize, value: i32 },
    PutVar { register: usize, var_id: usize, name: String },
    GetConst { register: usize, value: i32 },
    GetVar { register: usize, var_id: usize, name: String },
    Call { predicate: String },
    Proceed,
    Choice { alternative: usize },
    Allocate { n: usize },
    Deallocate,
    ArithmeticIs { target: usize, expression: Expression },
    SetLocal { index: usize, value: Term },
    GetLocal { index: usize, register: usize },
    Fail,
    GetStructure { register: usize, functor: String, arity: usize },
    IndexedCall { predicate: String, index_register: usize },
    MultiIndexedCall { predicate: String, index_registers: Vec<usize> },
    TailCall { predicate: String },
    AssertClause { predicate: String, address: usize },
    RetractClause { predicate: String, address: usize },
    Cut,
    BuildCompound { target: usize, functor: String, arg_registers: Vec<usize> },
    PutStr { register: usize, value: String },
    GetStr { register: usize, value: String },
    Halt,
}

impl Instruction {
    /// Executes this instruction on the provided machine.
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
            Instruction::Halt => Ok(()),
        }
    }
}
