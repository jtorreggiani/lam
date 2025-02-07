// src/languages/prolog/interpreter.rs

use std::env;
use std::fs;
use std::collections::HashMap;

// Import Prolog parser and AST.
use lam::languages::prolog::parser::PrologParser;
use lam::languages::prolog::ast::{PrologClause, PrologGoal, PrologTerm};

// Import LAM machine modules.
use lam::machine::instruction::Instruction;
use lam::machine::core::Machine;

/// Scans the generated instructions to compute the number of registers needed.
fn compute_required_registers(instructions: &[Instruction]) -> usize {
    let max_reg = instructions.iter().fold(0, |acc, instr| {
        let reg = match instr {
            Instruction::PutConst { register, .. } => *register,
            Instruction::PutVar { register, .. } => *register,
            Instruction::GetConst { register, .. } => *register,
            Instruction::GetVar { register, .. } => *register,
            Instruction::ArithmeticIs { target, .. } => *target,
            Instruction::SetLocal { index, .. } => *index,
            Instruction::GetLocal { register, .. } => *register,
            Instruction::GetStructure { register, .. } => *register,
            Instruction::IndexedCall { index_register, .. } => *index_register,
            Instruction::MultiIndexedCall { index_registers, .. } => {
                *index_registers.iter().max().unwrap_or(&0)
            },
            Instruction::BuildCompound { target, arg_registers, .. } => {
                let max_arg = arg_registers.iter().max().cloned().unwrap_or(0);
                std::cmp::max(*target, max_arg)
            },
            Instruction::PutStr { register, .. } => *register,
            Instruction::GetStr { register, .. } => *register,
            _ => 0,
        };
        std::cmp::max(acc, reg)
    });
    max_reg + 1
}

/// Compiles the head of a clause into LAM instructions.
/// **Changed:** For every variable in the head we now use a GetVar instruction,
/// so that the incoming query’s argument will unify with the head.
fn compile_head(goal: &PrologGoal, reg_offset: usize, var_map: &mut HashMap<String, usize>) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    for (i, arg) in goal.args.iter().enumerate() {
        let reg = reg_offset + i;
        match arg {
            PrologTerm::Const(n) => {
                instructions.push(Instruction::GetConst { register: reg, value: *n });
            },
            PrologTerm::Str(s) => {
                instructions.push(Instruction::GetStr { register: reg, value: s.clone() });
            },
            PrologTerm::Atom(a) => {
                instructions.push(Instruction::GetStr { register: reg, value: a.clone() });
            },
            PrologTerm::Var(name) => {
                if let Some(&existing_reg) = var_map.get(name) {
                    // Always write the value into the current (designated) register.
                    instructions.push(Instruction::GetVar { register: reg, var_id: existing_reg, name: name.clone() });
                } else {
                    var_map.insert(name.clone(), reg);
                    instructions.push(Instruction::GetVar { register: reg, var_id: reg, name: name.clone() });
                }
            },
            PrologTerm::Compound(op, args) if op == "-" && args.len() == 2 => {
                // Process the left subterm into register reg_offset.
                match &args[0] {
                    PrologTerm::Const(n) => {
                        instructions.push(Instruction::GetConst { register: reg_offset, value: *n });
                    },
                    PrologTerm::Str(s) => {
                        instructions.push(Instruction::GetStr { register: reg_offset, value: s.clone() });
                    },
                    PrologTerm::Atom(a) => {
                        instructions.push(Instruction::GetStr { register: reg_offset, value: a.clone() });
                    },
                    PrologTerm::Var(name) => {
                        if let Some(&r) = var_map.get(name) {
                            instructions.push(Instruction::GetVar { register: reg_offset, var_id: r, name: name.clone() });
                        } else {
                            var_map.insert(name.clone(), reg_offset);
                            instructions.push(Instruction::GetVar { register: reg_offset, var_id: reg_offset, name: name.clone() });
                        }
                    },
                    _ => {},
                }
                // Process the right subterm into register reg_offset+1.
                match &args[1] {
                    PrologTerm::Const(n) => {
                        instructions.push(Instruction::GetConst { register: reg_offset+1, value: *n });
                    },
                    PrologTerm::Str(s) => {
                        instructions.push(Instruction::GetStr { register: reg_offset+1, value: s.clone() });
                    },
                    PrologTerm::Atom(a) => {
                        instructions.push(Instruction::GetStr { register: reg_offset+1, value: a.clone() });
                    },
                    PrologTerm::Var(name) => {
                        if let Some(&r) = var_map.get(name) {
                            instructions.push(Instruction::GetVar { register: reg_offset+1, var_id: r, name: name.clone() });
                        } else {
                            var_map.insert(name.clone(), reg_offset+1);
                            instructions.push(Instruction::GetVar { register: reg_offset+1, var_id: reg_offset+1, name: name.clone() });
                        }
                    },
                    _ => {},
                }
                instructions.push(Instruction::BuildCompound {
                    target: reg,
                    functor: "-".to_string(),
                    arg_registers: vec![reg_offset, reg_offset+1],
                });
            },
            _ => {},
        }
    }
    instructions
}

/// Compiles a goal (for the clause body or directives) into LAM instructions.
fn compile_goal(goal: &PrologGoal, reg_offset: usize, var_map: &mut HashMap<String, usize>) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    for (i, arg) in goal.args.iter().enumerate() {
        let reg = reg_offset + i;
        match arg {
            PrologTerm::Const(n) => {
                instructions.push(Instruction::PutConst { register: reg, value: *n });
            },
            PrologTerm::Str(s) => {
                instructions.push(Instruction::PutStr { register: reg, value: s.clone() });
            },
            PrologTerm::Atom(a) => {
                instructions.push(Instruction::PutStr { register: reg, value: a.clone() });
            },
            PrologTerm::Var(name) => {
                if let Some(&existing_reg) = var_map.get(name) {
                    // Always copy the value into the designated register.
                    instructions.push(Instruction::GetVar { register: reg, var_id: existing_reg, name: name.clone() });
                } else {
                    var_map.insert(name.clone(), reg);
                    instructions.push(Instruction::PutVar { register: reg, var_id: reg, name: name.clone() });
                }
            },
            PrologTerm::Compound(op, args) if op == "-" && args.len() == 2 => {
                // Process the left subterm into register reg_offset.
                match &args[0] {
                    PrologTerm::Const(n) => {
                        instructions.push(Instruction::PutConst { register: reg_offset, value: *n });
                    },
                    PrologTerm::Str(s) => {
                        instructions.push(Instruction::PutStr { register: reg_offset, value: s.clone() });
                    },
                    PrologTerm::Atom(a) => {
                        instructions.push(Instruction::PutStr { register: reg_offset, value: a.clone() });
                    },
                    PrologTerm::Var(name) => {
                        if let Some(&r) = var_map.get(name) {
                            instructions.push(Instruction::GetVar { register: reg_offset, var_id: r, name: name.clone() });
                        } else {
                            var_map.insert(name.clone(), reg_offset);
                            instructions.push(Instruction::PutVar { register: reg_offset, var_id: reg_offset, name: name.clone() });
                        }
                    },
                    _ => {},
                }
                // Process the right subterm into register reg_offset+1.
                match &args[1] {
                    PrologTerm::Const(n) => {
                        instructions.push(Instruction::PutConst { register: reg_offset+1, value: *n });
                    },
                    PrologTerm::Str(s) => {
                        instructions.push(Instruction::PutStr { register: reg_offset+1, value: s.clone() });
                    },
                    PrologTerm::Atom(a) => {
                        instructions.push(Instruction::PutStr { register: reg_offset+1, value: a.clone() });
                    },
                    PrologTerm::Var(name) => {
                        if let Some(&r) = var_map.get(name) {
                            instructions.push(Instruction::GetVar { register: reg_offset+1, var_id: r, name: name.clone() });
                        } else {
                            var_map.insert(name.clone(), reg_offset+1);
                            instructions.push(Instruction::PutVar { register: reg_offset+1, var_id: reg_offset+1, name: name.clone() });
                        }
                    },
                    _ => {},
                }
                instructions.push(Instruction::BuildCompound {
                    target: reg,
                    functor: "-".to_string(),
                    arg_registers: vec![reg_offset, reg_offset+1],
                });
            },
            _ => {},
        }
    }
    match goal.predicate.as_str() {
        "write" => instructions.push(Instruction::Call { predicate: "write".to_string() }),
        "nl" => instructions.push(Instruction::Call { predicate: "nl".to_string() }),
        "halt" => instructions.push(Instruction::Halt),
        "fail" => instructions.push(Instruction::Fail),
        _ => instructions.push(Instruction::Call { predicate: goal.predicate.clone() }),
    }
    instructions
}

/// Compiles a clause (fact or rule) into LAM instructions.
/// The head and body share a variable map.
fn compile_clause(clause: &PrologClause, reg_offset: usize) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut var_map = HashMap::new();
    // Compile the head.
    let head_insts = compile_head(&clause.head, reg_offset, &mut var_map);
    instructions.extend(head_insts);
    // Compile the body (if any) using the same register base for all goals.
    if let Some(body) = &clause.body {
        for goal in body {
            let insts = compile_goal(goal, reg_offset, &mut var_map);
            instructions.extend(insts);
        }
    }
    instructions.push(Instruction::Proceed);
    instructions
}

/// Compiles an entire Prolog program into three parts:
/// 1. Code for predicate definitions,
/// 2. A predicate table mapping predicate names to code addresses,
/// 3. Directive code to be executed immediately.
fn compile_program(clauses: Vec<PrologClause>) -> (Vec<Instruction>, HashMap<String, Vec<usize>>, Vec<Instruction>) {
    let mut pred_code = Vec::new();
    let mut pred_table: HashMap<String, Vec<usize>> = HashMap::new();
    let mut directive_code = Vec::new();
    let reg_offset = 0;
    for clause in clauses {
        if clause.head.predicate == "directive" {
            if let Some(body) = &clause.body {
                let mut var_map = HashMap::new();
                for goal in body {
                    let insts = compile_goal(goal, reg_offset, &mut var_map);
                    directive_code.extend(insts);
                }
            }
        } else {
            let addr = pred_code.len();
            pred_table.entry(clause.head.predicate.clone()).or_insert(Vec::new()).push(addr);
            let insts = compile_clause(&clause, reg_offset);
            pred_code.extend(insts);
        }
    }
    (pred_code, pred_table, directive_code)
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin prolog <prolog_file.pl>");
        return;
    }
    let filename = &args[1];
    let program_str = fs::read_to_string(filename).expect("Failed to read file");

    // Parse the Prolog program.
    let mut parser = PrologParser::new(&program_str);
    let clauses = parser.parse_program().expect("Failed to parse program");

    // Compile the program.
    let (pred_code, pred_table, directive_code) = compile_program(clauses);
    let mut code = pred_code;
    let directive_start = code.len();
    code.extend(directive_code);

    // Compute how many registers are needed.
    let computed_registers = compute_required_registers(&code);
    let num_registers = computed_registers.max(100); // Ensure at least 100 registers.

    let mut machine = Machine::new(num_registers, code);
    machine.verbose = true;

    // Register predicate clauses.
    for (pred, addresses) in pred_table {
        for addr in addresses {
            machine.register_predicate(pred.clone(), addr);
        }
    }

    // Set the program counter to the directive code (if any).
    if directive_start < machine.code.len() {
        machine.pc = directive_start;
    }

    if let Err(e) = machine.run() {
        eprintln!("Error during execution: {}", e);
    }
}
