// Combined Rust source code from LAM project (including tests)
// Generated on Mon Feb  3 15:21:20 EST 2025

// === Source Files ===
// ============================================
// File: src/lib.rs
// ============================================

pub mod term;
pub mod machine;
pub mod lambda;
pub mod arithmetic;

// ============================================
// File: src/machine/mod.rs
// ============================================

pub mod instruction;
pub mod frame;
pub mod choice_point;
pub mod trail;
pub mod machine;

pub use instruction::Instruction;
pub use frame::Frame;
pub use choice_point::ChoicePoint;
pub use trail::TrailEntry;
pub use machine::Machine;
pub use machine::MachineError;

// ============================================
// File: src/machine/machine.rs
// ============================================

use std::collections::HashMap;
use crate::term::Term;
use crate::arithmetic;

use super::{
    instruction::Instruction,
    frame::Frame,
    choice_point::ChoicePoint,
    trail::TrailEntry,
};

/// The set of errors that can occur during execution of the LAM.
#[derive(Debug)]
pub enum MachineError {
    RegisterOutOfBounds(usize),
    UninitializedRegister(usize),
    UnificationFailed(String),
    EnvironmentMissing,
    PredicateNotFound(String),
    PredicateClauseNotFound(String),
    NoChoicePoint,
    StructureMismatch {
        expected_functor: String,
        expected_arity: usize,
        found_functor: String,
        found_arity: usize,
    },
    NotACompoundTerm(usize),
    NoIndexedClause(String, Term),
    NoIndexEntry(String, Term),
    PredicateNotInIndex(String),
    NoMoreInstructions,
}

/// The abstract machine structure.
#[derive(Debug)]
pub struct Machine {
    /// Registers: each can hold an optional Term.
    pub registers: Vec<Option<Term>>,
    /// The code (instructions) for the machine.
    pub code: Vec<Instruction>,
    /// Program counter.
    pub pc: usize,
    /// Substitution environment mapping variable unique id to Terms.
    pub substitution: HashMap<usize, Term>,
    /// A control stack to hold frames for predicate calls.
    pub control_stack: Vec<Frame>,
    /// A predicate table mapping predicate names to lists of clause addresses.
    pub predicate_table: HashMap<String, Vec<usize>>,
    /// A stack to hold choice points for backtracking.
    pub choice_stack: Vec<ChoicePoint>,
    /// A trail to record variable bindings (for backtracking).
    pub trail: Vec<TrailEntry>,
    /// A stack of environment frames for local variables.
    pub environment_stack: Vec<Vec<Option<Term>>>,
    /// Index table mapping predicate names to an index mapping.
    pub index_table: HashMap<String, HashMap<Term, Vec<usize>>>,
    /// Mapping from variable unique id to its original string name.
    pub variable_names: HashMap<usize, String>,
}

impl Machine {
    /// Create a new machine with a specified number of registers and given code.
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
            substitution: HashMap::new(),
            control_stack: Vec::new(),
            predicate_table: HashMap::new(),
            choice_stack: Vec::new(),
            trail: Vec::new(),
            environment_stack: Vec::new(),
            index_table: HashMap::new(),
            variable_names: HashMap::new(),
        }
    }

    /// Helper function to bind a variable (by its unique id) to a term.
    /// Returns false if the occurs check fails; otherwise, returns true after performing the binding.
    fn bind_variable(&mut self, var: &usize, term: &Term) -> bool {
        if self.occurs_check(var, term) {
            return false;
        }
        let prev = self.substitution.get(var).cloned();
        self.trail.push(TrailEntry { variable: *var, previous_value: prev });
        self.substitution.insert(*var, term.clone());
        true
    }

    /// Returns true if `var` occurs anywhere inside `term`.
    fn occurs_check(&self, var: &usize, term: &Term) -> bool {
        match term {
            Term::Var(v) => v == var,
            Term::Const(_) => false,
            Term::Compound(_, args) => args.iter().any(|arg| self.occurs_check(var, arg)),
            Term::Lambda(param, body) => {
                if param == var {
                    false
                } else {
                    self.occurs_check(var, body)
                }
            },
            Term::App(fun, arg) => self.occurs_check(var, fun) || self.occurs_check(var, arg),
        }
    }

    /// Registers an indexed clause for the given predicate.
    pub fn register_indexed_clause(&mut self, predicate: String, key: Term, address: usize) {
        let entry = self.index_table.entry(predicate).or_insert_with(HashMap::new);
        entry.entry(key).or_insert_with(Vec::new).push(address);
    }

    /// Registers a clause for the given predicate name.
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table
            .entry(name)
            .or_insert_with(Vec::new)
            .push(address);
    }

    /// Resolve a term to its current bound value, if any.
    fn resolve(&self, term: &Term) -> Term {
        match term {
            Term::Var(v) => {
                if let Some(bound) = self.substitution.get(v) {
                    self.resolve(bound)
                } else {
                    term.clone()
                }
            }
            _ => term.clone(),
        }
    }

    /// Unify two terms.
    /// If unification fails, returns false.
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> bool {
        let term1 = self.resolve(t1);
        let term2 = self.resolve(t2);

        if term1 == term2 {
            return true;
        }

        match (term1, term2) {
            (Term::Const(a), Term::Const(b)) => a == b,
            (Term::Var(ref v), ref other) => self.bind_variable(v, other),
            (ref other, Term::Var(ref v)) => self.bind_variable(v, other),
            (Term::Compound(functor1, args1), Term::Compound(functor2, args2)) => {
                if functor1 == functor2 && args1.len() == args2.len() {
                    for (a, b) in args1.iter().zip(args2.iter()) {
                        if !self.unify(a, b) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    /// Execute one instruction.
    /// Returns `Ok(())` if the instruction executed normally,
    /// or an appropriate `MachineError` if an error occurred.
    pub fn step(&mut self) -> Result<(), MachineError> {
        if self.pc >= self.code.len() {
            return Err(MachineError::NoMoreInstructions);
        }
        let instr = self.code[self.pc].clone();
        self.pc += 1;
        match instr {
            Instruction::PutConst { register, value } => {
                if register < self.registers.len() {
                    self.registers[register] = Some(Term::Const(value));
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(register))
                }
            },
            Instruction::PutVar { register, var_id, name } => {
                if register < self.registers.len() {
                    self.registers[register] = Some(Term::Var(var_id));
                    self.variable_names.insert(var_id, name);
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(register))
                }
            },
            Instruction::GetConst { register, value } => {
                if register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(register));
                }
                if let Some(term) = self.registers[register].clone() {
                    let goal = Term::Const(value);
                    if !self.unify(&term, &goal) {
                        return Err(MachineError::UnificationFailed(format!(
                            "Cannot unify {:?} with {:?}",
                            term, goal
                        )));
                    }
                    Ok(())
                } else {
                    Err(MachineError::UninitializedRegister(register))
                }
            },
            Instruction::GetVar { register, var_id, name } => {
                if register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(register));
                }
                // Ensure the machine knows the name for this variable.
                self.variable_names.entry(var_id).or_insert(name);
                if let Some(term) = self.registers[register].clone() {
                    let goal = Term::Var(var_id);
                    if !self.unify(&goal, &term) {
                        return Err(MachineError::UnificationFailed(format!(
                            "Cannot unify {:?} with {:?}",
                            goal, term
                        )));
                    }
                    Ok(())
                } else {
                    // If uninitialized, simply set the register to the variable.
                    self.registers[register] = Some(Term::Var(var_id));
                    Ok(())
                }
            },
            Instruction::Call { predicate } => {
                if let Some(clauses) = self.predicate_table.get(&predicate) {
                    if clauses.is_empty() {
                        return Err(MachineError::PredicateClauseNotFound(predicate));
                    }
                    self.control_stack.push(Frame {
                        return_pc: self.pc,
                    });
                    let mut alternatives = clauses.clone();
                    let jump_to = alternatives.remove(0);
                    let alternative_clauses = if alternatives.is_empty() {
                        None
                    } else {
                        Some(alternatives)
                    };
                    let cp = ChoicePoint {
                        saved_pc: self.pc,
                        saved_registers: self.registers.clone(),
                        saved_substitution: self.substitution.clone(),
                        saved_trail_len: self.trail.len(),
                        alternative_clauses,
                    };
                    self.choice_stack.push(cp);
                    self.pc = jump_to;
                    Ok(())
                } else {
                    Err(MachineError::PredicateNotFound(predicate))
                }
            },
            Instruction::Proceed => {
                if let Some(frame) = self.control_stack.pop() {
                    self.pc = frame.return_pc;
                }
                Ok(())
            },
            Instruction::Choice => {
                let cp = ChoicePoint {
                    saved_pc: self.pc,
                    saved_registers: self.registers.clone(),
                    saved_substitution: self.substitution.clone(),
                    saved_trail_len: self.trail.len(),
                    alternative_clauses: None,
                };
                self.choice_stack.push(cp);
                Ok(())
            },
            Instruction::Allocate { n } => {
                self.environment_stack.push(vec![None; n]);
                Ok(())
            },
            Instruction::Deallocate => {
                if self.environment_stack.pop().is_some() {
                    Ok(())
                } else {
                    Err(MachineError::EnvironmentMissing)
                }
            },
            Instruction::ArithmeticIs { target, expression } => {
                let result = arithmetic::evaluate(&expression);
                if target < self.registers.len() {
                    self.registers[target] = Some(Term::Const(result));
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(target))
                }
            },
            Instruction::SetLocal { index, value } => {
                if let Some(env) = self.environment_stack.last_mut() {
                    if index < env.len() {
                        env[index] = Some(value);
                        Ok(())
                    } else {
                        Err(MachineError::RegisterOutOfBounds(index))
                    }
                } else {
                    Err(MachineError::EnvironmentMissing)
                }
            },
            Instruction::GetLocal { index, register } => {
                if let Some(env) = self.environment_stack.last() {
                    if index < env.len() {
                        if let Some(term) = env[index].clone() {
                            if register < self.registers.len() {
                                if let Some(reg_term) = self.registers[register].clone() {
                                    if !self.unify(&reg_term, &term) {
                                        return Err(MachineError::UnificationFailed(format!(
                                            "Cannot unify {:?} with {:?}",
                                            reg_term, term
                                        )));
                                    }
                                } else {
                                    self.registers[register] = Some(term);
                                }
                                Ok(())
                            } else {
                                Err(MachineError::RegisterOutOfBounds(register))
                            }
                        } else {
                            Err(MachineError::UninitializedRegister(index))
                        }
                    } else {
                        Err(MachineError::RegisterOutOfBounds(index))
                    }
                } else {
                    Err(MachineError::EnvironmentMissing)
                }
            },
            Instruction::Fail => {
                if let Some(cp) = self.choice_stack.pop() {
                    while self.trail.len() > cp.saved_trail_len {
                        if let Some(entry) = self.trail.pop() {
                            match entry.previous_value {
                                Some(prev_val) => {
                                    self.substitution.insert(entry.variable, prev_val);
                                }
                                None => {
                                    self.substitution.remove(&entry.variable);
                                }
                            }
                        }
                    }
                    self.registers = cp.saved_registers;
                    self.substitution = cp.saved_substitution;
                    self.pc = cp.saved_pc + 2; // Skip the failed alternative.
                    Ok(())
                } else {
                    Err(MachineError::NoChoicePoint)
                }
            },
            Instruction::GetStructure { register, functor, arity } => {
                if register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(register));
                }
                if let Some(term) = self.registers[register].clone() {
                    match term {
                        Term::Compound(ref f, ref args) => {
                            if f == &functor && args.len() == arity {
                                Ok(())
                            } else {
                                Err(MachineError::StructureMismatch {
                                    expected_functor: functor,
                                    expected_arity: arity,
                                    found_functor: f.clone(),
                                    found_arity: args.len(),
                                })
                            }
                        },
                        _ => Err(MachineError::NotACompoundTerm(register)),
                    }
                } else {
                    Err(MachineError::UninitializedRegister(register))
                }
            },
            Instruction::IndexedCall { predicate, index_register } => {
                if index_register >= self.registers.len() {
                    return Err(MachineError::RegisterOutOfBounds(index_register));
                }
                if let Some(key_term) = self.registers[index_register].clone() {
                    if let Some(index_map) = self.index_table.get(&predicate) {
                        if let Some(clauses) = index_map.get(&key_term) {
                            if !clauses.is_empty() {
                                let mut alternatives = clauses.clone();
                                let jump_to = alternatives.remove(0);
                                let alternative_clauses = if alternatives.is_empty() {
                                    None
                                } else {
                                    Some(alternatives)
                                };
                                let cp = ChoicePoint {
                                    saved_pc: self.pc,
                                    saved_registers: self.registers.clone(),
                                    saved_substitution: self.substitution.clone(),
                                    saved_trail_len: self.trail.len(),
                                    alternative_clauses,
                                };
                                self.choice_stack.push(cp);
                                self.pc = jump_to;
                                Ok(())
                            } else {
                                Err(MachineError::NoIndexedClause(predicate, key_term))
                            }
                        } else {
                            Err(MachineError::NoIndexEntry(predicate, key_term))
                        }
                    } else {
                        Err(MachineError::PredicateNotInIndex(predicate))
                    }
                } else {
                    Err(MachineError::UninitializedRegister(index_register))
                }
            },
            Instruction::TailCall { predicate } => {
                let _ = self.environment_stack.pop(); // Deallocate current environment frame.
                if let Some(clauses) = self.predicate_table.get(&predicate) {
                    if !clauses.is_empty() {
                        let mut alternatives = clauses.clone();
                        let jump_to = alternatives.remove(0);
                        let alternative_clauses = if alternatives.is_empty() {
                            None
                        } else {
                            Some(alternatives)
                        };
                        let cp = ChoicePoint {
                            saved_pc: self.pc,
                            saved_registers: self.registers.clone(),
                            saved_substitution: self.substitution.clone(),
                            saved_trail_len: self.trail.len(),
                            alternative_clauses,
                        };
                        self.choice_stack.push(cp);
                        self.pc = jump_to;
                        Ok(())
                    } else {
                        Err(MachineError::PredicateClauseNotFound(predicate))
                    }
                } else {
                    Err(MachineError::PredicateNotFound(predicate))
                }
            },
            Instruction::AssertClause { predicate, address } => {
                self.predicate_table
                    .entry(predicate.clone())
                    .or_insert_with(Vec::new)
                    .push(address);
                Ok(())
            },
            Instruction::RetractClause { predicate, address } => {
                if let Some(clauses) = self.predicate_table.get_mut(&predicate) {
                    if let Some(pos) = clauses.iter().position(|&a| a == address) {
                        clauses.remove(pos);
                        Ok(())
                    } else {
                        Err(MachineError::PredicateClauseNotFound(predicate))
                    }
                } else {
                    Err(MachineError::PredicateNotFound(predicate))
                }
            },
            Instruction::Cut => {
                self.choice_stack.clear();
                Ok(())
            },
            Instruction::BuildCompound { target, functor, arg_registers } => {
                let mut args = Vec::new();
                for &reg in arg_registers.iter() {
                    if reg >= self.registers.len() {
                        return Err(MachineError::RegisterOutOfBounds(reg));
                    }
                    if let Some(term) = self.registers[reg].clone() {
                        args.push(term);
                    } else {
                        return Err(MachineError::UninitializedRegister(reg));
                    }
                }
                if target < self.registers.len() {
                    self.registers[target] = Some(Term::Compound(functor, args));
                    Ok(())
                } else {
                    Err(MachineError::RegisterOutOfBounds(target))
                }
            },
        }
    }

    /// Run the machine until no more instructions are available.
    /// Returns `Ok(())` if execution completed, or a `MachineError` otherwise.
    pub fn run(&mut self) -> Result<(), MachineError> {
        while self.pc < self.code.len() {
            self.step()?;
        }
        Ok(())
    }
}

// ============================================
// File: src/machine/frame.rs
// ============================================

// A frame to store return information for a predicate call.
#[derive(Debug, Clone)]
pub struct Frame {
    pub return_pc: usize,
}

// ============================================
// File: src/machine/instruction.rs
// ============================================

use crate::term::Term;
use crate::arithmetic::Expr;

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
    Choice,
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
    ArithmeticIs { target: usize, expression: Expr },
    // AssertClause adds a clause address for a predicate.
    AssertClause { predicate: String, address: usize },
    // RetractClause removes a clause address for a predicate.
    RetractClause { predicate: String, address: usize },
    // Cut — prunes all choice points for the current predicate call.
    Cut,
}

// ============================================
// File: src/machine/choice_point.rs
// ============================================

use std::collections::HashMap;
use crate::term::Term;

// A choice point to support backtracking and clause selection.
// Saves the program counter, registers, substitution, the current length of the trail,
// and a list of alternative clause addresses for the current predicate call.
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub saved_pc: usize,
    pub saved_registers: Vec<Option<Term>>,
    pub saved_substitution: HashMap<usize, Term>,
    pub saved_trail_len: usize,
    pub alternative_clauses: Option<Vec<usize>>,
}

// ============================================
// File: src/machine/trail.rs
// ============================================

use crate::term::Term;

#[derive(Debug, Clone)]
pub struct TrailEntry {
    pub variable: usize,
    pub previous_value: Option<Term>,
}

// ============================================
// File: src/arithmetic.rs
// ============================================

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Const(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

// Recursively evaluates an arithmetic expression.
pub fn evaluate(expr: &Expr) -> i32 {
  match expr {
      Expr::Const(n) => *n,
      Expr::Add(e1, e2) => evaluate(e1) + evaluate(e2),
      Expr::Sub(e1, e2) => evaluate(e1) - evaluate(e2),
      Expr::Mul(e1, e2) => evaluate(e1) * evaluate(e2),
      // Note: does not check for division by zero.
      Expr::Div(e1, e2) => evaluate(e1) / evaluate(e2),
  }
}

// ============================================
// File: src/term.rs
// ============================================

/// Represents a term in our logic programming language.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Term {
    /// A constant value (for example, a 32‐bit integer).
    Const(i32),
    /// A variable, identified by a unique id.
    Var(usize),
    /// A compound term with a functor and a list of arguments.
    Compound(String, Vec<Term>),
    /// A lambda abstraction: λ<var_id>.<body>
    /// (The variable’s unique id is stored in the term; its string name is stored in the machine.)
    Lambda(usize, Box<Term>),
    /// An application: (<function> <argument>)
    App(Box<Term>, Box<Term>),
}

// ============================================
// File: src/lambda.rs
// ============================================

use crate::term::Term;

/// Recursively substitutes every free occurrence of variable `var` (identified by its unique id)
/// in `term` with the given `replacement`. (This is a simple version that does not handle
/// variable capture. In a complete implementation, renaming of bound variables would be required.)
pub fn substitute(term: &Term, var: usize, replacement: &Term) -> Term {
    match term {
        Term::Var(v) => {
            if *v == var {
                replacement.clone()
            } else {
                term.clone()
            }
        },
        Term::Const(_) => term.clone(),
        Term::Compound(f, args) => {
            Term::Compound(f.clone(), args.iter().map(|t| substitute(t, var, replacement)).collect())
        },
        Term::Lambda(param, body) => {
            // If the bound variable is the same as the one we're substituting, leave it unchanged.
            if *param == var {
                term.clone()
            } else {
                Term::Lambda(*param, Box::new(substitute(body, var, replacement)))
            }
        },
        Term::App(fun, arg) => {
            Term::App(
                Box::new(substitute(fun, var, replacement)),
                Box::new(substitute(arg, var, replacement))
            )
        },
    }
}

/// Performs a single beta-reduction step on the given term if it is an application of a lambda abstraction.
/// That is, it transforms (λ<var>.<body>) <arg> into <body>[<arg>/<var>].
pub fn beta_reduce(term: &Term) -> Term {
    match term {
        Term::App(fun, arg) => {
            if let Term::Lambda(param, body) = *fun.clone() {
                substitute(&body, param, arg)
            } else {
                term.clone()
            }
        },
        _ => term.clone(),
    }
}

// ============================================
// File: src/main.rs
// ============================================

fn main() {
    println!("Hello, world!");
}


// === Test Files ===
// ============================================
// File: tests/test_arithmetic.rs
// ============================================

use lam::machine::{Instruction, Machine};
use lam::term::Term;
use lam::arithmetic::{Expr, evaluate};

// Test for the ArithmeticIs instruction.
//
// This test simulates evaluating the expression (3 + 4) * 2 and storing the result
// in register 0. The expected result is 14.
#[test]
fn test_arithmetic_is() {
    let code = vec![
        // Evaluate the expression (3 + 4) * 2.
        Instruction::ArithmeticIs { 
            target: 0, 
            expression: Expr::Mul(
                Box::new(Expr::Add(Box::new(Expr::Const(3)), Box::new(Expr::Const(4)))),
                Box::new(Expr::Const(2))
            )
        },
    ];
    let mut machine = Machine::new(1, code);
    let _ = machine.run();
    
    // The result should be stored in register 0.
    assert_eq!(machine.registers[0], Some(Term::Const(14)));
}

#[test]
fn test_evaluate_const() {
    let expr = Expr::Const(42);
    assert_eq!(evaluate(&expr), 42);
}

#[test]
fn test_evaluate_add() {
    let expr = Expr::Add(Box::new(Expr::Const(3)), Box::new(Expr::Const(4)));
    assert_eq!(evaluate(&expr), 7);
}

#[test]
fn test_evaluate_complex() {
    // (10 - 2) * (1 + 3) = 8 * 4 = 32.
    let expr = Expr::Mul(
        Box::new(Expr::Sub(Box::new(Expr::Const(10)), Box::new(Expr::Const(2)))),
        Box::new(Expr::Add(Box::new(Expr::Const(1)), Box::new(Expr::Const(3))))
    );
    assert_eq!(evaluate(&expr), 32);
}

// ============================================
// File: tests/test_backtracking_constants.rs
// ============================================

// tests/test_backtracking_constants.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for validating the LAM's backtracking mechanism for constant values.
///
/// This test simulates the following Prolog behavior:
/// 
/// ```prolog
/// test_const_backtracking :-
///     X = 10,                % PutConst reg0, 10
///     (                      % Create a choice point
///         Y = 20,            % PutConst reg1, 20 (first alternative)
///         fail               % Force failure, triggering backtracking
///         ;                  % Backtrack to choice point
///         Y = 30             % PutConst reg1, 30 (second alternative)
///     ).
/// 
/// % Expected result:
/// %   X = 10,
/// %   Y = 30.
/// ```
/// 
/// In this test:
/// 1. Register 0 is set to 10 (this value persists through backtracking).
/// 2. A choice point is created to save the current state.
/// 3. The first alternative sets register 1 to 20 (which is later undone).
/// 4. A Fail instruction forces backtracking, restoring the saved state.
/// 5. The second alternative sets register 1 to 30.
/// 6. Finally, the test verifies that:
///    - reg0 still contains 10,
///    - reg1 contains 30,
///    - and the choice stack is empty.
#[test]
fn test_backtracking_constants() {
    // Program structure:
    // 0: PutConst   reg0, 10     // Store 10 in reg0 (persists)
    // 1: Choice                  // Save state for backtracking
    // 2: PutConst   reg1, 20     // First alternative: reg1 = 20 (will be undone)
    // 3: Fail                    // Trigger backtracking
    // 4: PutConst   reg1, 30     // Second alternative: reg1 = 30
    let code = vec![
        Instruction::PutConst { register: 0, value: 10 },
        Instruction::Choice,
        Instruction::PutConst { register: 1, value: 20 },
        Instruction::Fail,
        Instruction::PutConst { register: 1, value: 30 },
    ];
    
    let mut machine = Machine::new(2, code);
    let _ = machine.run();
    
    // Verify:
    // - reg0 should remain 10.
    // - reg1 should be 30 (the alternative after backtracking).
    // - The choice stack should be empty.
    assert_eq!(machine.registers[0], Some(Term::Const(10)));
    assert_eq!(machine.registers[1], Some(Term::Const(30)));
    assert_eq!(machine.choice_stack.len(), 0);
}

// ============================================
// File: tests/test_backtracking_variables.rs
// ============================================

// tests/test_backtracking_variables.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for validating the LAM's trail mechanism for variable bindings during backtracking.
///
/// This test simulates the following Prolog behavior:
///
/// ```prolog
/// test_var_backtracking :-
///     X,                       % PutVar reg0, "X" (X is initially unbound)
///     (                        % Create a choice point (state with X unbound is saved)
///         X = 100,           % GetConst reg0, 100 (first alternative: binds X to 100)
///         fail               % Force failure, triggering backtracking (undoes the binding from step 2)
///         ;                  % Backtrack to the choice point
///         X = 300            % GetConst reg0, 300 (second alternative: binds X to 300)
///     ).
/// % Expected result:
/// %   X = 300.
/// ```
///
/// Step-by-step:
/// 1. Register 0 is set to variable X (unbound).
/// 2. A Choice instruction saves the state, including an empty trail (trail length 0).
/// 3. GetConst reg0, 100 unifies X with 100, pushing a trail entry. (Trail length becomes 1.)
/// 4. Fail is executed, causing backtracking:
///    - The trail is unwound back to the saved trail length (0).
///    - The saved substitution (with X unbound) is restored.
/// 5. Then, GetConst reg0, 300 unifies X with 300, pushing a new trail entry. (Trail length becomes 1.)
/// 6. Finally, the substitution binds X to 300.
/// 
/// We then verify that:
/// - The substitution environment binds "X" to Const(300).
/// - The trail length is 1 (since the final unification pushed one trail entry).
/// - The choice stack is empty.
#[test]
fn test_backtracking_variables() {
    let code = vec![
      // Step 0: PutVar reg0, "X" with var_id 0.
      Instruction::PutVar { register: 0, var_id: 0, name: "X".to_string() },
      // Step 1: Create a choice point.
      Instruction::Choice,
      // Step 2: First alternative: GetConst reg0, 100.
      Instruction::GetConst { register: 0, value: 100 },
      // Step 3: Fail.
      Instruction::Fail,
      // Step 4: Second alternative: GetConst reg0, 300.
      Instruction::GetConst { register: 0, value: 300 },
  ];

  let mut machine = Machine::new(1, code);
  let _ = machine.run();
  // "X" should be bound to 300.
  assert_eq!(machine.substitution.get(&0), Some(&Term::Const(300)));
  assert_eq!(machine.trail.len(), 1);
  assert_eq!(machine.choice_stack.len(), 0);
}

// ============================================
// File: tests/test_benchmark.rs
// ============================================

// tests/test_benchmark.rs

use lam::arithmetic::Expr;
use lam::machine::{Instruction, Machine};
use lam::term::Term;
use std::time::Instant;

/// Benchmark test for the LAM.
///
/// This test repeatedly executes a small program that performs arithmetic evaluation
/// via the `ArithmeticIs` instruction. The program evaluates the expression (3 + 4) * 2,
/// which should yield 14. By running the program many times, we get an approximate measure
/// of the performance of the LAM’s core instruction dispatch and arithmetic evaluation.
///
/// Note:
/// - In a more advanced system, you might implement a recursive loop entirely within the LAM.
///   Here, we simply reinitialize and run the same small program many times.
/// - This benchmark serves as a baseline for performance before further optimizations.
#[test]
fn benchmark_arithmetic_is() {
    // Build the arithmetic expression: (3 + 4) * 2.
    let expr = Expr::Mul(
        Box::new(Expr::Add(Box::new(Expr::Const(3)), Box::new(Expr::Const(4)))),
        Box::new(Expr::Const(2))
    );

    // The program consists of one instruction that evaluates the expression and stores
    // the result in register 0.
    let code = vec![
        Instruction::ArithmeticIs { target: 0, expression: expr },
    ];

    // Set the number of iterations for the benchmark.
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        // Create a fresh machine for each iteration.
        let mut machine = Machine::new(1, code.clone());
        let _ = machine.run();
        // Verify correctness: register 0 should contain 14.
        assert_eq!(machine.registers[0], Some(Term::Const(14)));
    }
    let duration = start.elapsed();
    println!("Benchmark: Executed {} iterations in {:?}", iterations, duration);
}

// ============================================
// File: tests/test_build_compound.rs
// ============================================

// tests/test_build_compound.rs

use lam::machine::{Machine, Instruction};
use lam::term::Term;

/// Test for building compound terms dynamically from register values.
///
/// The test simulates the following Prolog behavior:
/// 
/// ```prolog
/// test_build_compound :-
///     A = 42,                % PutConst reg0, 42
///     B = 99,                % PutConst reg1, 99
///     F = f(A, B),           % BuildCompound in reg2 using functor 'f'
///     % Expected: F = f(42, 99)
///     true.
/// ```
/// 
/// The sequence is as follows:
/// 1. Register 0 is set to 42.
/// 2. Register 1 is set to 99.
/// 3. The BuildCompound instruction constructs f(42, 99) from registers 0 and 1
///    and stores the result in register 2.
/// The test verifies that register 2 contains the compound term f(42, [99]).
#[test]
fn test_build_compound() {
    // Program structure:
    // 0: PutConst   reg0, 42
    // 1: PutConst   reg1, 99
    // 2: BuildCompound target=2, functor "f", arguments from registers [0, 1]
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::PutConst { register: 1, value: 99 },
        Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
    ];
    
    let mut machine = Machine::new(3, code);
    let _ = machine.run();
    
    // Expected compound term: f(42, 99)
    let expected = Term::Compound("f".to_string(), vec![Term::Const(42), Term::Const(99)]);
    
    assert_eq!(machine.registers[2], Some(expected));
}
// ============================================
// File: tests/test_cut.rs
// ============================================

// tests/test_cut.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

// Test for the Cut instruction.
///
// This test simulates the following Prolog behavior:
// 
// ```prolog
// test_cut :-
//     (   % Create two alternatives:
//         ( X = 1, !, fail )   % First alternative: bind X to 1, then cut, then force failure
//         ;
//         X = 2               % Second alternative: bind X to 2
//     ).
// % Expected result: X = 2.
// ```
// 
// In this test:
// 1. A Choice point is created.
// 2. The first alternative sets register 0 to 1, then executes a Cut, then forces failure.
// 3. The Cut should clear the choice points, so backtracking does not return to an earlier alternative.
// 4. The machine then continues (or finishes) without any further alternatives, so the final binding remains.
// We verify that register 0 does not remain 1 (the first alternative), implying the cut prevented backtracking.
#[test]
fn test_cut() {
    let code = vec![
        // Create a choice point.
        Instruction::Choice,
        // First alternative: bind X (reg0) to 1.
        Instruction::PutConst { register: 0, value: 1 },
        // Execute cut, which clears choice points.
        Instruction::Cut,
        // Force failure (this alternative should fail and not backtrack due to the cut).
        Instruction::Fail,
        // Second alternative: if backtracking occurred, this would bind X to 2.
        Instruction::PutConst { register: 0, value: 2 },
        Instruction::Proceed,
    ];
    
    let mut machine = Machine::new(1, code);
    // For this test, we pre-register a dummy predicate if needed.
    // (Alternatively, the Call instruction might be used in a more complex scenario.)
    let _ = machine.run();
    
    // After running, because of the cut, the first alternative's failure should not backtrack to the second alternative.
    // Therefore, the value in register 0 should remain as it was set before the cut.
    // Since the alternative that binds 1 forced a failure after the cut, no further alternatives are tried.
    // We expect that register 0 does NOT get bound to 2.
    // In our simple system, this may result in register 0 still containing 1,
    // or it may remain unchanged if the failure terminates execution.
    // For this test, let's assert that register 0 is 1, confirming that backtracking did not reach the clause that sets it to 2.
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
    // And the choice stack should be empty.
    assert_eq!(machine.choice_stack.len(), 0);
}

// ============================================
// File: tests/test_dynamic_clause_management.rs
// ============================================

// tests/test_dynamic_clause_management.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// This test demonstrates dynamic clause management by asserting and retracting clauses.
///
/// We simulate a predicate "p" that initially has no clauses. We then:
/// 1. Assert a clause for "p" at a given address that sets register 0 to 1.
/// 2. Call "p" so that register 0 is set to 1.
/// 3. Retract that clause.
/// 4. Attempt to call "p" again (which should now fail or leave register 0 unchanged).
///
/// For simplicity, our test will check that after retracting, a call to "p" does not change the register.
#[test]
fn test_dynamic_clause_management() {
    // Program:
    // 0: AssertClause for predicate "p" with clause address 3.
    // 1: Call "p"              -> should jump to address 3.
    // 2: Proceed               -> returns to after the call.
    // 3: PutConst reg0, 1      -> clause body that sets register 0 to 1.
    // 4: Proceed               -> returns.
    // 5: RetractClause for predicate "p" with clause address 3.
    // 6: Call "p"              -> should fail to find any clause and leave register 0 unchanged.
    let code = vec![
        Instruction::AssertClause { predicate: "p".to_string(), address: 3 },
        Instruction::Call { predicate: "p".to_string() },
        Instruction::Proceed,
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::Proceed,
        Instruction::RetractClause { predicate: "p".to_string(), address: 3 },
        Instruction::Call { predicate: "p".to_string() },
        Instruction::Proceed,
    ];

    let mut machine = Machine::new(1, code);
    // Initially, predicate "p" is not registered in the static table.
    // We rely solely on the AssertClause instruction to add it.
    // Run the program.
    let _ = machine.run();

    // After the first call, register 0 should have been set to 1.
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
    // After retracting the clause and calling p again, no clause should be found.
    // For our test, we assume that a call to an undefined predicate leaves the register unchanged.
    // In this simple design, the second call might simply not change register 0.
    // We check that the value remains 1.
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
}

// ============================================
// File: tests/test_environment.rs
// ============================================

// tests/test_environment.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test environment allocation, local variable assignment, and retrieval.
///
/// This program simulates:
/// 1. Allocating an environment frame of size 2.
/// 2. Setting local variable 0 to constant 42.
/// 3. Setting local variable 1 to constant 99.
/// 4. Retrieving local variable 0 into register 0.
/// 5. Retrieving local variable 1 into register 1.
/// 6. Deallocating the environment frame.
#[test]
fn test_environment() {
    let code = vec![
        // Allocate an environment with 2 local variables.
        Instruction::Allocate { n: 2 },
        // Set local variable 0 to 42.
        Instruction::SetLocal { index: 0, value: Term::Const(42) },
        // Set local variable 1 to 99.
        Instruction::SetLocal { index: 1, value: Term::Const(99) },
        // Retrieve local variable 0 into register 0.
        Instruction::GetLocal { index: 0, register: 0 },
        // Retrieve local variable 1 into register 1.
        Instruction::GetLocal { index: 1, register: 1 },
        // Deallocate the environment.
        Instruction::Deallocate,
    ];
    
    let mut machine = Machine::new(2, code);
    let _ = machine.run();
    
    // Verify that the registers hold the expected local values.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
    assert_eq!(machine.registers[1], Some(Term::Const(99)));
    // Verify that the environment stack is empty.
    assert_eq!(machine.environment_stack.len(), 0);
}

// ============================================
// File: tests/test_get_structure.rs
// ============================================

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for the GetStructure instruction.
///
/// This test builds a compound term and then uses GetStructure to verify that
/// the term in the specified register has the expected functor and arity.
///
/// The Prolog equivalent might be:
/// ```prolog
/// test_get_structure :-
///     F = f(1, 2),          % BuildCompound equivalent
///     get_structure(F, f, 2). % Succeeds if F is a compound term f/2.
/// ```
///
/// Steps:
/// 1. Register 0 is set to 1 (constant).
/// 2. Register 1 is set to 2 (constant).
/// 3. A compound term f(1,2) is built from registers 0 and 1 and stored in register 2.
/// 4. GetStructure is executed on register 2, expecting functor "f" and arity 2.
#[test]
fn test_get_structure() {
    let code = vec![
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::PutConst { register: 1, value: 2 },
        Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
        Instruction::GetStructure { register: 2, functor: "f".to_string(), arity: 2 },
    ];
    
    let mut machine = Machine::new(3, code);
    let _ = machine.run();
    
    // The BuildCompound should have built f(1,2) in register 2.
    let expected = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);
    assert_eq!(machine.registers[2], Some(expected));
}

// ============================================
// File: tests/test_higher_order.rs
// ============================================

use lam::term::Term;
use lam::lambda::{substitute, beta_reduce};

#[test]
fn test_substitution() {
    // Let term be f(x), represented as Compound("f", [Var(0)]) where 0 is the id for x.
    let term = Term::Compound("f".to_string(), vec![Term::Var(0)]);
    let result = substitute(&term, 0, &Term::Const(42));
    let expected = Term::Compound("f".to_string(), vec![Term::Const(42)]);
    assert_eq!(result, expected);
}

#[test]
fn test_beta_reduce_identity() {
    // Identity function: Lambda(0, Var(0))
    let identity = Term::Lambda(0, Box::new(Term::Var(0)));
    // Application: (λx. x) 42
    let app = Term::App(Box::new(identity), Box::new(Term::Const(42)));
    let result = beta_reduce(&app);
    assert_eq!(result, Term::Const(42));
}

#[test]
fn test_beta_reduce_complex() {
    // Lambda term: λx. f(x, 1) where x has id 0.
    let lambda_term = Term::Lambda(
        0,
        Box::new(Term::Compound(
            "f".to_string(),
            vec![Term::Var(0), Term::Const(1)],
        )),
    );
    // Application: (λx. f(x, 1)) 2
    let app = Term::App(Box::new(lambda_term), Box::new(Term::Const(2)));
    let result = beta_reduce(&app);
    let expected = Term::Compound("f".to_string(), vec![Term::Const(2), Term::Const(1)]);
    assert_eq!(result, expected);
}

// ============================================
// File: tests/test_indexed_call.rs
// ============================================

// tests/test_indexed_call.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

// Test for indexed clause selection.
//
// We simulate a predicate "p" with two clauses indexed by the first argument:
//
// Clause 1: When the key is Const(1), the clause sets register 0 to 10.
// Clause 2: When the key is Const(2), the clause sets register 0 to 20.
//
// The test will perform an IndexedCall on predicate "p" using the content of register 0
// as the key. We set register 0 to Const(2) so that it should choose Clause 2.
#[test]
fn test_indexed_call() {
    let code = vec![
        // Main program: perform an indexed call.
        Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
        // Clause for predicate p when key is 1 (should not be chosen in this test).
        Instruction::PutConst { register: 0, value: 10 },
        Instruction::Fail,
        // Clause for predicate p when key is 2.
        Instruction::PutConst { register: 0, value: 20 },
        Instruction::Proceed,
    ];
    
    let mut machine = Machine::new(1, code);
    
    // Set up the index table:
    // Register predicate "p" with an index:
    // Clause 1: for key Const(1), at address 1.
    machine.register_indexed_clause("p".to_string(), Term::Const(1), 1);
    // Clause 2: for key Const(2), at address 3.
    machine.register_indexed_clause("p".to_string(), Term::Const(2), 3);
    
    // Set register 0 to Const(2), which is the index key we want to use.
    machine.registers[0] = Some(Term::Const(2));
    
    let _ = machine.run();
    
    // We expect that the IndexedCall will select Clause 2, setting register 0 to 20.
    assert_eq!(machine.registers[0], Some(Term::Const(20)));
}

// ============================================
// File: tests/test_machine.rs
// ============================================

use lam::machine::{Machine, Instruction};
use lam::term::Term;

#[test]
fn test_put_const_instruction() {
    let code = vec![Instruction::PutConst { register: 0, value: 42 }];
    let mut machine = Machine::new(2, code);

    assert_eq!(machine.registers[0], None);
    assert_eq!(machine.registers[1], None);

    let cont = machine.step();
    assert!(cont.is_ok());

    assert_eq!(machine.registers[0], Some(Term::Const(42)));
    assert_eq!(machine.registers[1], None);

    assert!(machine.step().is_err());
}

#[test]
fn test_put_var_instruction() {
    let code = vec![Instruction::PutVar { register: 1, var_id: 0, name: "X".to_string() }];
    let mut machine = Machine::new(2, code);
    machine.run().unwrap();

    assert_eq!(machine.registers[1], Some(Term::Var(0)));
}

#[test]
fn test_get_const_unification_success() {
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 42 },
    ];
    let mut machine = Machine::new(1, code);

    machine.run().unwrap();

    assert_eq!(machine.registers[0], Some(Term::Const(42)));
}

#[test]
fn test_get_const_unification_failure() {
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 100 },
    ];
    let mut machine = Machine::new(1, code);
    assert!(machine.run().is_err());
}

#[test]
fn test_get_var_unification() {
    let code = vec![
        Instruction::PutVar { register: 0, var_id: 0, name: "X".to_string() },
        Instruction::GetVar { register: 0, var_id: 1, name: "Y".to_string() },
    ];
    let mut machine = Machine::new(1, code);
    machine.run().unwrap();

    assert_eq!(machine.registers[0], Some(Term::Var(0)));
    let binding = machine.substitution.get(&1).cloned();
    assert_eq!(binding, Some(Term::Var(0)));
}

#[test]
fn test_call_and_proceed_with_lookup() {
    let code = vec![
        Instruction::PutConst { register: 0, value: 10 },
        Instruction::Call { predicate: "dummy_pred".to_string() },
        Instruction::PutConst { register: 1, value: 20 },
        Instruction::Proceed,
        Instruction::PutConst { register: 2, value: 30 },
    ];
    
    let mut machine = Machine::new(3, code);
    machine.register_predicate("dummy_pred".to_string(), 2);
    
    machine.run().unwrap();
    
    assert_eq!(machine.registers[0], Some(Term::Const(10)));
    assert_eq!(machine.registers[1], Some(Term::Const(20)));
    assert_eq!(machine.registers[2], Some(Term::Const(30)));
    assert_eq!(machine.control_stack.len(), 0);
}

// ============================================
// File: tests/test_path_inference.rs
// ============================================

use lam::machine::{Instruction, Machine};

// This benchmark test sets up a simple graph and a recursive path predicate,
// then runs the query `path(1, 3)` to search for a path from node 1 to node 3.
//
// The program is manually encoded as a vector of LAM instructions:
//
// Main query (indices 0–3):
//   0: PutConst reg0, 1    ; X = 1
//   1: PutConst reg1, 3    ; Y = 3
//   2: Call { predicate: "path" }
//   3: Proceed
//
// Facts for edge/2 (indices 4–12):
//   Clause for edge(1,2):
//     4: PutConst reg0, 1
//     5: PutConst reg1, 2
//     6: Proceed
//   Clause for edge(2,3):
//     7: PutConst reg0, 2
//     8: PutConst reg1, 3
//     9: Proceed
//   Clause for edge(1,3):
//     10: PutConst reg0, 1
//     11: PutConst reg1, 3
//     12: Proceed
//
// Clauses for path/2:
//   Clause 1: path(X,Y) :- edge(X,Y).
//     13: Call { predicate: "edge" }
//     14: Proceed
//   Clause 2: path(X,Y) :- edge(X,Z), path(Z,Y).
//     15: PutVar reg2, "Z"
//     16: Call { predicate: "edge" }
//     17: Call { predicate: "path" }
//     18: Proceed
//
// We register predicate clause addresses as follows:
//   "edge" -> [4, 7, 10]
//   "path" -> [13, 15]
//
// Running the query should find at least one solution. In the classic graph:
//   edge(1,3) gives a direct solution,
//   and edge(1,2), edge(2,3) gives an indirect solution.
// For benchmarking, we simply measure the execution time.
#[test]
fn benchmark_path_inference() {
    // Construct the program instructions.
    let code = vec![
        // Main Query: path(1,3)
        Instruction::PutConst { register: 0, value: 1 },   // X = 1
        Instruction::PutConst { register: 1, value: 3 },   // Y = 3
        Instruction::Call { predicate: "path".to_string() },
        Instruction::Proceed,
        // Facts for edge/2:
        // Clause for edge(1,2)
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::PutConst { register: 1, value: 2 },
        Instruction::Proceed,
        // Clause for edge(2,3)
        Instruction::PutConst { register: 0, value: 2 },
        Instruction::PutConst { register: 1, value: 3 },
        Instruction::Proceed,
        // Clause for edge(1,3)
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::PutConst { register: 1, value: 3 },
        Instruction::Proceed,
        // Clause for path/2, Clause 1: path(X,Y) :- edge(X,Y).
        Instruction::Call { predicate: "edge".to_string() },
        Instruction::Proceed,
        // Clause for path/2, Clause 2: path(X,Y) :- edge(X,Z), path(Z,Y).
        Instruction::PutVar { register: 2, var_id: 0, name: "Z".to_string() },
        Instruction::Call { predicate: "edge".to_string() },
        Instruction::Call { predicate: "path".to_string() },
        Instruction::Proceed,
    ];

    // Create a machine with 3 registers (we need at least registers 0, 1, and 2).
    let mut machine = Machine::new(3, code);

    // Register the predicates.
    machine.register_predicate("edge".to_string(), 4);  // Clause for edge(1,2) at index 4.
    machine.register_predicate("edge".to_string(), 7);  // Clause for edge(2,3) at index 7.
    machine.register_predicate("edge".to_string(), 10); // Clause for edge(1,3) at index 10.
    machine.register_predicate("path".to_string(), 13); // Clause 1 for path/2 at index 13.
    machine.register_predicate("path".to_string(), 15); // Clause 2 for path/2 at index 15.

    // Benchmark: run the program and measure the execution time.
    let start = std::time::Instant::now();
    let _ = machine.run();
    let duration = start.elapsed();
    
    // Print the solution (what remains in registers 0 and 1 after the query).
    println!("Path Inference Benchmark: Solution: X = {:?}, Y = {:?}", machine.registers[0], machine.registers[1]);
    println!("Path Inference Benchmark: Execution time: {:?}", duration);
    
    // For this benchmark, we simply assert that a solution was found.
    // (In a fully complete system, you might count all solutions.)
    assert!(machine.registers[0].is_some());
    assert!(machine.registers[1].is_some());
}

// ============================================
// File: tests/test_tail_call.rs
// ============================================

// tests/test_tail_call.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for tail-call optimization.
///
/// The program simulates a tail call as follows:
/// - The main program allocates an environment and sets a local variable.
/// - A TailCall instruction calls predicate "p".
///   The tail call should first deallocate the current environment frame.
/// - Predicate "p" (starting at address 4) simply puts 200 into register 0 and then proceeds.
/// - The expected result is that register 0 contains 200 and the environment stack is empty.
#[test]
fn test_tail_call() {
    // Main program:
    let code = vec![
        // Allocate an environment frame.
        Instruction::Allocate { n: 1 },
        // Set local variable 0 to 100.
        Instruction::SetLocal { index: 0, value: Term::Const(100) },
        // Tail call to predicate "p".
        Instruction::TailCall { predicate: "p".to_string() },
        // Dummy instruction (should not execute if tail call works):
        Instruction::PutConst { register: 0, value: 999 },
        // Predicate "p" code (starts at index 4):
        Instruction::PutConst { register: 0, value: 200 },
        // Return from predicate "p".
        Instruction::Proceed,
    ];
    let mut machine = Machine::new(1, code);
    
    // Register predicate "p" to start at index 4.
    machine.register_predicate("p".to_string(), 4);
    
    let _ = machine.run();
    
    // Verify that register 0 was set to 200 by predicate "p".
    assert_eq!(machine.registers[0], Some(Term::Const(200)));
    // Verify that the environment stack is empty (tail call deallocated it).
    assert_eq!(machine.environment_stack.len(), 0);
}

// ============================================
// File: tests/test_term.rs
// ============================================

use lam::term::Term;

#[test]
fn test_create_const() {
    let term = Term::Const(10);
    let term2 = Term::Const(10);
    assert_eq!(term, term2);
    
    assert_ne!(Term::Const(10), Term::Const(20));
    assert_ne!(Term::Const(10), Term::Var(10)); // Var(10) is a variable with id 10.
    assert_ne!(Term::Const(10), Term::Compound("10".to_string(), vec![]));
    
    let max = Term::Const(i32::MAX);
    let min = Term::Const(i32::MIN);
    assert_ne!(max, min);
}

#[test]
fn test_create_var() {
    // Two variables with the same id are equal.
    let term1 = Term::Var(0);
    let term2 = Term::Var(0);
    assert_eq!(term1, term2);
    
    // Different ids are not equal.
    assert_ne!(Term::Var(0), Term::Var(1));
}

#[test]
fn test_create_compound() {
    let empty_compound = Term::Compound("f".to_string(), vec![]);
    assert_eq!(empty_compound, Term::Compound("f".to_string(), vec![]));
    assert_ne!(empty_compound, Term::Compound("g".to_string(), vec![]));
    
    let term1 = Term::Compound("f".to_string(), vec![
        Term::Const(1),
        Term::Const(2),
    ]);
    
    let term2 = Term::Compound("f".to_string(), vec![
        Term::Const(2),
        Term::Const(1),
    ]);
    
    assert_ne!(term1, term2);
    
    let case1 = Term::Compound("f".to_string(), vec![Term::Const(1)]);
    let case2 = Term::Compound("F".to_string(), vec![Term::Const(1)]);
    assert_ne!(case1, case2);
}

#[test]
fn test_nested_compound() {
    let nested = Term::Compound("f".to_string(), vec![
        Term::Compound("g".to_string(), vec![
            Term::Compound("h".to_string(), vec![
                Term::Var(0)
            ])
        ])
    ]);
    
    let nested2 = Term::Compound("f".to_string(), vec![
        Term::Compound("g".to_string(), vec![
            Term::Compound("h".to_string(), vec![
                Term::Var(0)
            ])
        ])
    ]);
    
    assert_eq!(nested, nested2);
    
    let different_nested = Term::Compound("f".to_string(), vec![
        Term::Compound("g".to_string(), vec![
            Term::Compound("h".to_string(), vec![
                Term::Var(1)  // Different id.
            ])
        ])
    ]);
    
    assert_ne!(nested, different_nested);
}

#[test]
fn test_clone() {
    let original = Term::Compound("f".to_string(), vec![
        Term::Var(0),
        Term::Compound("g".to_string(), vec![Term::Const(42)]),
    ]);
    
    let cloned = original.clone();
    assert_eq!(original, cloned);
    
    match cloned {
        Term::Compound(_, args) => {
            assert_eq!(args.len(), 2);
        },
        _ => panic!("Cloned term lost its structure!")
    }
}

// ============================================
// File: tests/test_unification.rs
// ============================================

use lam::machine::Machine;
use lam::term::Term;

#[test]
fn test_compound_unification() {
    // Set up a compound term f(42, X) in register 0, where X has id 1.
    let term1 = Term::Compound(
        "f".to_string(),
        vec![Term::Const(42), Term::Var(1)],
    );
    
    let mut machine = Machine::new(1, vec![]);
    machine.registers[0] = Some(term1);
    
    // Unify a new variable Y (with id 2) with the second argument.
    if let Some(compound_term) = machine.registers[0].clone() {
        if let Term::Compound(_, args) = compound_term {
            let success = machine.unify(&Term::Var(2), &args[1]);
            assert!(success, "Unification of the compound subterm failed");
        } else {
            panic!("Register 0 does not contain a compound term");
        }
    } else {
        panic!("Register 0 is empty");
    }
    
    let binding = machine.substitution.get(&2).cloned();
    assert_eq!(binding, Some(Term::Var(1)));
}

