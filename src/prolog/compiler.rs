// src/prolog/compiler.rs
//! A minimal Prolog compiler that compiles a Prolog file (facts and rules)
//! into a LAM program. It parses all clauses (ignoring query lines) and
//! compiles each clause into a code block. It also builds a predicate table.
//!
//! Facts are compiled into a code block that uses PUT instructions to record
//! each argument (if any) so that the fact’s head is fully represented.
//!
//! Rules are compiled by processing each goal (the body of the clause).
//!
//! For goals that are equality predicates (i.e. with functor "=" and two arguments)
//! are handled specially. For goals with functor "write", we now check whether the argument
//! is an atom or a variable. If it is a variable, we emit a MOVE instruction to move the variable's
//! value from register 1 into register 0 before CALLing "write".
//! For all other compound goals, the compiler generates PUT instructions for each argument
//! (placing atoms, numbers, or variables into registers) before emitting a CALL instruction.
//!
//! Finally, a PROCEED is appended at the end of each clause’s code block.

use std::collections::HashMap;
use std::error::Error;
use crate::machine::instruction::Instruction;
use crate::prolog::ast::{Clause, Term};
use crate::prolog::parser::{parse_program};

/// Compiles a Prolog program (facts and rules) into a LAM program.
/// Returns a tuple of (compiled instructions, predicate table).
///
/// The predicate table maps each predicate name (from the clause head) to a list of
/// starting addresses for the corresponding code blocks.
pub fn compile_prolog(program: &str) -> Result<(Vec<Instruction>, HashMap<String, Vec<usize>>), Box<dyn Error>> {
    let clauses = parse_program(program)
        .map_err(|e| Box::<dyn Error>::from(format!("Parse error: {:?}", e)))?;
    
    let mut instructions = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();
    
    for clause in clauses {
        let addr = instructions.len();
        let (pred_name, code_block) = match clause {
            Clause::Fact { head } => {
                // For a fact, if the head is a compound term, generate PUT instructions for each argument.
                match head {
                    Term::Compound(functor, args) => {
                        let mut block = Vec::new();
                        for (i, arg) in args.iter().enumerate() {
                            match arg {
                                Term::Atom(s) => {
                                    block.push(Instruction::PutStr { register: i, value: s.clone() });
                                },
                                Term::Number(n) => {
                                    block.push(Instruction::PutConst { register: i, value: *n });
                                },
                                Term::Var(s) => {
                                    block.push(Instruction::PutVar { register: i, var_id: i, name: s.clone() });
                                },
                                _ => return Err(Box::from("Unsupported argument type in fact head")),
                            }
                        }
                        block.push(Instruction::Proceed);
                        (functor, block)
                    },
                    Term::Atom(s) => {
                        // Fact with no arguments.
                        (s, vec![Instruction::Proceed])
                    },
                    _ => return Err(Box::from("Unsupported fact head type")),
                }
            },
            Clause::Rule { head, body } => {
                // The head’s predicate name (we ignore its arguments during call compilation).
                let name = match head {
                    Term::Atom(s) => s,
                    Term::Compound(ref functor, _) => functor.clone(),
                    _ => return Err(Box::from("Unsupported rule head type")),
                };
                let mut block = Vec::new();
                for goal in body {
                    match goal {
                        // Special handling for unification goal: X = Y
                        Term::Compound(ref functor, ref args) if functor == "=" && args.len() == 2 => {
                            // Move left argument into register 0
                            match args[0] {
                                Term::Atom(ref s) => {
                                    block.push(Instruction::PutStr { register: 0, value: s.clone() });
                                },
                                Term::Number(n) => {
                                    block.push(Instruction::PutConst { register: 0, value: n });
                                },
                                Term::Var(ref s) => {
                                    block.push(Instruction::PutVar { register: 0, var_id: 0, name: s.clone() });
                                },
                                _ => return Err(Box::from("Unsupported argument type for equality (left)")),
                            }
                            // Move right argument into register 1
                            match args[1] {
                                Term::Atom(ref s) => {
                                    block.push(Instruction::PutStr { register: 1, value: s.clone() });
                                },
                                Term::Number(n) => {
                                    block.push(Instruction::PutConst { register: 1, value: n });
                                },
                                Term::Var(ref s) => {
                                    block.push(Instruction::PutVar { register: 1, var_id: 1, name: s.clone() });
                                },
                                _ => return Err(Box::from("Unsupported argument type for equality (right)")),
                            }
                            // Emit call to built–in "="
                            block.push(Instruction::Call { predicate: "=".to_string() });
                        },
                        // Special handling for write goal.
                        Term::Compound(ref functor, ref args) if functor == "write" && args.len() == 1 => {
                            match &args[0] {
                                Term::Atom(ref literal) => {
                                    // If the argument is a literal, simply put it in register 0.
                                    block.push(Instruction::PutStr { register: 0, value: literal.clone() });
                                    block.push(Instruction::Call { predicate: functor.clone() });
                                },
                                Term::Var(_) => {
                                    // For a variable, we assume that the variable was already bound
                                    // from a previous goal (e.g. in a call to parent) and is in register 1.
                                    // So we emit a MOVE instruction to move the value from register 1 to register 0.
                                    block.push(Instruction::Move { src: 1, dst: 0 });
                                    block.push(Instruction::Call { predicate: functor.clone() });
                                },
                                _ => {
                                    // Fallback: compile generically.
                                    for (i, arg) in args.iter().enumerate() {
                                        match arg {
                                            Term::Atom(ref s) => { block.push(Instruction::PutStr { register: i, value: s.clone() }); },
                                            Term::Number(n) => { block.push(Instruction::PutConst { register: i, value: *n }); },
                                            Term::Var(ref s) => { block.push(Instruction::PutVar { register: i, var_id: i, name: s.clone() }); },
                                            _ => return Err(Box::from("Unsupported argument type in goal")),
                                        }
                                    }
                                    block.push(Instruction::Call { predicate: functor.clone() });
                                },
                            }
                        },
                        // For any other compound goal, generate PUT instructions for each argument.
                        Term::Compound(ref functor, ref args) => {
                            for (i, arg) in args.iter().enumerate() {
                                match arg {
                                    Term::Atom(ref s) => {
                                        block.push(Instruction::PutStr { register: i, value: s.clone() });
                                    },
                                    Term::Number(n) => {
                                        block.push(Instruction::PutConst { register: i, value: *n });
                                    },
                                    Term::Var(ref s) => {
                                        block.push(Instruction::PutVar { register: i, var_id: i, name: s.clone() });
                                    },
                                    _ => return Err(Box::from("Unsupported argument type in goal")),
                                }
                            }
                            block.push(Instruction::Call { predicate: functor.clone() });
                        },
                        // For an atomic goal (with no arguments), simply emit a CALL.
                        Term::Atom(s) => {
                            block.push(Instruction::Call { predicate: s });
                        },
                        _ => return Err(Box::from("Unsupported goal type in rule")),
                    }
                }
                block.push(Instruction::Proceed);
                (name, block)
            },
        };
        instructions.extend(code_block);
        predicate_table.entry(pred_name).or_insert_with(Vec::new).push(addr);
    }
    
    Ok((instructions, predicate_table))
}
