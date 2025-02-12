// src/prolog/compiler.rs
//! A minimal Prolog compiler that compiles a Prolog file (facts and rules)
//! into a LAM program. It parses all clauses (ignoring query lines) and
//! compiles each clause into a code block. It also builds a predicate table.
//!
//! Facts are compiled into a single instruction (Proceed).
//! Rules are compiled by processing each goal in the body:
//!   - If the goal is a compound term with functor "=" and two arguments,
//!     it compiles it as a unification goal by moving the left and right arguments
//!     into registers 0 and 1 respectively, and then emitting a Call to the built–in "=".
//!   - If the goal is a compound term with functor "write" and one argument,
//!     it emits a PutStr instruction to load the string into register 0 and then a Call for "write".
//!   - Otherwise, if the goal is a compound term or an atom, it emits a Call.
//!
//! Finally, a Proceed is appended at the end of each clause's code block.
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
                let name = match head {
                    Term::Atom(s) => s,
                    Term::Compound(ref functor, _) => functor.clone(),
                    _ => return Err(Box::from("Unsupported fact head type")),
                };
                (name, vec![Instruction::Proceed])
            },
            Clause::Rule { head, body } => {
                let name = match head {
                    Term::Atom(s) => s,
                    Term::Compound(ref functor, _) => functor.clone(),
                    _ => return Err(Box::from("Unsupported rule head type")),
                };
                let mut block = Vec::new();
                for goal in body {
                    match goal {
                        // Handle unification goal: X = Y
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
                                    // Use var_id 0 for the left argument.
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
                                    // Use var_id 1 for the right argument.
                                    block.push(Instruction::PutVar { register: 1, var_id: 1, name: s.clone() });
                                },
                                _ => return Err(Box::from("Unsupported argument type for equality (right)")),
                            }
                            // Emit call to built-in "="
                            block.push(Instruction::Call { predicate: "=".to_string() });
                        },
                        Term::Compound(ref functor, ref args) if functor == "write" && args.len() == 1 => {
                            if let Term::Atom(ref literal) = args[0] {
                                block.push(Instruction::PutStr { register: 0, value: literal.clone() });
                                block.push(Instruction::Call { predicate: functor.clone() });
                            } else {
                                block.push(Instruction::Call { predicate: functor.clone() });
                            }
                        },
                        Term::Compound(ref functor, _) => {
                            block.push(Instruction::Call { predicate: functor.clone() });
                        },
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
