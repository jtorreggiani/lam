// src/languages/prolog/interpreter.rs
//! Prolog interpreter for the LAM system.
//! [Header omitted for brevity]

use std::env;
use std::fs;
use std::collections::HashMap;

use lam::languages::prolog::parser::PrologParser;
use lam::languages::prolog::ast::{PrologClause, PrologGoal, PrologTerm};

use lam::machine::instruction::Instruction;
use lam::machine::core::Machine;

/// Pre-scan a clause (head and body) and assign each variable a unique register number.
fn allocate_clause_vars(clause: &PrologClause) -> HashMap<String, usize> {
    let mut mapping = HashMap::new();
    fn scan_term(term: &PrologTerm, mapping: &mut HashMap<String, usize>, next_reg: &mut usize) {
        match term {
            PrologTerm::Var(name) => {
                if !mapping.contains_key(name) {
                    mapping.insert(name.clone(), *next_reg);
                    *next_reg += 1;
                }
            },
            PrologTerm::Compound(_, args) => {
                for arg in args {
                    scan_term(arg, mapping, next_reg);
                }
            },
            _ => {}
        }
    }
    let mut next_reg = 0;
    for arg in &clause.head.args {
        scan_term(arg, &mut mapping, &mut next_reg);
    }
    if let Some(goals) = &clause.body {
        for goal in goals {
            for arg in &goal.args {
                scan_term(arg, &mut mapping, &mut next_reg);
            }
        }
    }
    log::debug!("Allocated variable mapping for clause: {:?}", mapping);
    mapping
}

/// Helper: compile a term so that its value ends up in register `target`.
fn compile_term_to_reg(term: &PrologTerm, mapping: &HashMap<String, usize>, target: usize) -> Vec<Instruction> {
    match term {
        PrologTerm::Const(n) => vec![Instruction::PutConst { register: target, value: *n }],
        PrologTerm::Str(s) => vec![Instruction::PutStr { register: target, value: s.clone() }],
        PrologTerm::Atom(a) => vec![Instruction::PutStr { register: target, value: a.clone() }],
        PrologTerm::Var(name) => {
            let allocated = mapping.get(name).expect("Variable must have been allocated");
            if *allocated == target {
                vec![Instruction::GetVar { register: target, var_id: target, name: name.clone() }]
            } else {
                vec![Instruction::Move { src: *allocated, dst: target }]
            }
        },
        PrologTerm::Compound(op, args) if op == "-" && args.len() == 2 => {
            let mut instrs = Vec::new();
            instrs.extend(compile_term_to_reg(&args[0], mapping, target));
            instrs.extend(compile_term_to_reg(&args[1], mapping, target + 1));
            instrs.push(Instruction::BuildCompound { target, functor: "-".to_string(), arg_registers: vec![target, target+1] });
            instrs
        },
        _ => vec![],
    }
}

/// Compiles the head of a clause into LAM instructions.
/// The head is compiled so that its arguments end up in the environment registers
/// as given by the precomputed mapping.
fn compile_head(goal: &PrologGoal, mapping: &HashMap<String, usize>) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    // For each argument, force its value into the environment register (mapping[name]).
    for arg in &goal.args {
        match arg {
            PrologTerm::Const(n) => {
                // Find an environment register for constants by assigning it to a new register.
                // (For head matching, we assume constants are directly compared.)
                // Here we simply emit a GetConst into the register given by its order.
                // (If the head contains no variable, then the caller’s argument is a constant.)
                instructions.push(Instruction::GetConst { register: 0, value: *n });
            },
            PrologTerm::Str(s) => {
                instructions.push(Instruction::GetStr { register: 0, value: s.clone() });
            },
            PrologTerm::Atom(a) => {
                instructions.push(Instruction::GetStr { register: 0, value: a.clone() });
            },
            PrologTerm::Var(name) => {
                let reg = *mapping.get(name).expect("Variable must have been allocated");
                instructions.push(Instruction::GetVar { register: reg, var_id: reg, name: name.clone() });
            },
            PrologTerm::Compound(op, args) if op == "-" && args.len() == 2 => {
                instructions.extend(compile_term_to_reg(arg, mapping, 0));
            },
            _ => {}
        }
    }
    instructions
}

/// Compiles a goal (in the clause body) into LAM instructions using a temporary argument area.
/// The parameter `base` specifies the first register of the argument area.
/// After the call, for each argument that is a variable, we insert a Move to copy
/// the computed value from the temporary area into the environment register.
fn compile_goal(goal: &PrologGoal, mapping: &HashMap<String, usize>, base: usize) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    match goal.predicate.as_str() {
        // Built-in predicates with no argument
        "nl" | "halt" | "fail" => {
            instructions.push(Instruction::Call { predicate: goal.predicate.clone() });
        },
        "write" => {
            if goal.args.len() != 1 {
                panic!("write/1 expects exactly one argument");
            }
            // For write, if the argument is a variable, compile it into its environment register.
            let instrs = match &goal.args[0] {
                PrologTerm::Var(name) => {
                    let env_reg = *mapping.get(name).expect("Variable allocated");
                    compile_term_to_reg(&goal.args[0], mapping, env_reg)
                },
                _ => compile_term_to_reg(&goal.args[0], mapping, base)
            };
            instructions.extend(instrs);
            instructions.push(Instruction::Call { predicate: "write".to_string() });
        },
        _ => {
            // For a non-built-in predicate, compile each argument into the temporary area.
            for (i, arg) in goal.args.iter().enumerate() {
                instructions.extend(compile_term_to_reg(arg, mapping, base + i));
            }
            instructions.push(Instruction::Call { predicate: goal.predicate.clone() });
            // After the call, for each argument that is a variable, copy its value into the environment.
            for (i, arg) in goal.args.iter().enumerate() {
                if let PrologTerm::Var(name) = arg {
                    let env_reg = *mapping.get(name).expect("Variable allocated");
                    if env_reg != (base + i) {
                        instructions.push(Instruction::Move { src: base + i, dst: env_reg });
                    }
                }
            }
        }
    }
    instructions
}

/// Compiles a clause (fact or rule) into LAM instructions.
/// We pre-scan the clause for variables, then compile the head into the environment
/// and compile each goal in the body using a temporary argument area starting at a fixed offset.
/// (Here we choose 10 if there is at least one variable.)
fn compile_clause(clause: &PrologClause, _dummy: usize) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mapping = allocate_clause_vars(clause);
    // Determine environment size.
    let env_size = if mapping.is_empty() { 0 } else { 10 };
    // Compile the head (if any). For a directive clause the head is a dummy.
    let head_insts = compile_head(&clause.head, &mapping);
    instructions.extend(head_insts);
    if let Some(body) = &clause.body {
        for goal in body {
            let insts = compile_goal(goal, &mapping, env_size);
            instructions.extend(insts);
        }
    }
    instructions.push(Instruction::Proceed);
    log::debug!("Compiled clause {:?}: instructions: {:?}", clause, instructions);
    instructions
}

/// Compiles an entire Prolog program into three parts.
fn compile_program(clauses: Vec<PrologClause>) -> (Vec<Instruction>, HashMap<String, Vec<usize>>, Vec<Instruction>) {
    let mut pred_code = Vec::new();
    let mut pred_table: HashMap<String, Vec<usize>> = HashMap::new();
    let mut directive_code = Vec::new();
    for clause in clauses {
        if clause.head.predicate == "directive" {
            let insts = compile_clause(&clause, 0);
            directive_code.extend(insts);
        } else {
            let addr = pred_code.len();
            pred_table.entry(clause.head.predicate.clone()).or_insert(Vec::new()).push(addr);
            let insts = compile_clause(&clause, 0);
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
    let program_str = fs::read_to_string(filename)
        .expect(&format!("Failed to read file: {}", filename));
    let mut parser = PrologParser::new(&program_str);
    let clauses = parser.parse_program().expect("Failed to parse program");
    let (pred_code, pred_table, directive_code) = compile_program(clauses);
    let mut code = pred_code;
    let directive_start = code.len();
    code.extend(directive_code);
    // We choose a fixed register count (e.g. 100).
    let num_registers = 100;
    let mut machine = Machine::new(num_registers, code);
    machine.verbose = true;
    for (pred, addresses) in pred_table {
        for addr in addresses {
            machine.register_predicate(pred.clone(), addr);
        }
    }
    if directive_start < machine.code.len() {
        machine.pc = directive_start;
    }
    if let Err(e) = machine.run() {
        eprintln!("Error during execution: {}", e);
    }
}
