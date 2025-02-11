// src/machine/instruction.rs
//! Definitions of LAM instructions.
use std::fmt;

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
    /// New: Move instruction copies the content from register `src` into register `dst`.
    Move { src: usize, dst: usize },
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
            Instruction::Move { src, dst } => machine.execute_move(*src, *dst),
            Instruction::Halt => Ok(()),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::PutConst { register, value } =>
                write!(f, "PUT_CONST R{}, {}", register, value),
            Instruction::PutVar { register, var_id, name } =>
                write!(f, "PUT_VAR   R{}, {}, \"{}\"", register, var_id, name),
            Instruction::GetConst { register, value } =>
                write!(f, "GET_CONST R{}, {}", register, value),
            Instruction::GetVar { register, var_id, name } =>
                write!(f, "GET_VAR   R{}, {}, \"{}\"", register, var_id, name),
            Instruction::Call { predicate } =>
                write!(f, "CALL      \"{}\"", predicate),
            Instruction::Proceed =>
                write!(f, "PROCEED"),
            Instruction::Choice { alternative } =>
                write!(f, "CHOICE    {}", alternative),
            Instruction::Allocate { n } =>
                write!(f, "ALLOCATE  {}", n),
            Instruction::Deallocate =>
                write!(f, "DEALLOCATE"),
            Instruction::ArithmeticIs { target, expression } =>
                write!(f, "ARITHMETIC_IS R{}, {:?}", target, expression),
            Instruction::SetLocal { index, value } =>
                write!(f, "SET_LOCAL {} <- {:?}", index, value),
            Instruction::GetLocal { index, register } =>
                write!(f, "GET_LOCAL {} -> R{}", index, register),
            Instruction::Fail =>
                write!(f, "FAIL"),
            Instruction::GetStructure { register, functor, arity } =>
                write!(f, "GET_STRUCTURE R{}, {} / {}", register, functor, arity),
            Instruction::IndexedCall { predicate, index_register } =>
                write!(f, "INDEXED_CALL \"{}\", R{}", predicate, index_register),
            Instruction::MultiIndexedCall { predicate, index_registers } =>
                write!(f, "MULTI_INDEXED_CALL \"{}\", {:?}", predicate, index_registers),
            Instruction::TailCall { predicate } =>
                write!(f, "TAIL_CALL \"{}\"", predicate),
            Instruction::AssertClause { predicate, address } =>
                write!(f, "ASSERT_CLAUSE \"{}\", {}", predicate, address),
            Instruction::RetractClause { predicate, address } =>
                write!(f, "RETRACT_CLAUSE \"{}\", {}", predicate, address),
            Instruction::Cut =>
                write!(f, "CUT"),
            Instruction::BuildCompound { target, functor, arg_registers } =>
                write!(f, "BUILD_COMPOUND R{}, {} {:?}", target, functor, arg_registers),
            Instruction::PutStr { register, value } =>
                write!(f, "PUT_STR   R{}, \"{}\"", register, value),
            Instruction::GetStr { register, value } =>
                write!(f, "GET_STR   R{}, \"{}\"", register, value),
            Instruction::Move { src, dst } =>
                write!(f, "MOVE      R{} -> R{}", src, dst),
            Instruction::Halt =>
                write!(f, "HALT"),
        }
    }
}
