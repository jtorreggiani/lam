//! A Prolog interpreter built on top of the LAM abstract machine.
//!
//! This interpreter is designed to behave like SWI‑Prolog. It supports:
//!   • Queries entered at the prompt (e.g., `?- parent(john, X).`)
//!   • Dynamic clause management via built‑in predicates:
//!         assert(<clause>).
//!         retract(<clause>).
//!         :- dynamic(<pred>/<arity>).
//!
//! Facts (or clauses) and queries are compiled into LAM instructions using a minimal
//! compiler. The fact compiler uses Get‑instructions so that the clause head is used
//! for unification. (Rules are not supported in this minimal version.)
//!
//! Usage:
//!   • To load a Prolog source file: `cargo run --bin prolog myprogram.pl`
//!   • To start the interactive REPL: `cargo run --bin prolog`

use std::collections::HashMap;
use std::io::{self, Write};

use lam::machine::Machine;
use lam::machine::instruction::Instruction;
use lam::term::Term;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let content = std::fs::read_to_string(filename)
            .expect("Failed to read the source file");
        run_prolog_program(&content);
    } else {
        run_repl();
    }
}

/// Runs a Prolog program loaded from a source file.
/// The file may contain fact clauses, queries, and directives.
fn run_prolog_program(source: &str) {
    let mut db_code: Vec<Instruction> = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();
    let mut query_code: Option<Vec<Instruction>> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with(":- dynamic(") {
            // Acknowledge dynamic declarations.
            continue;
        } else if trimmed.starts_with("assert(") {
            // Process an assertion command.
            let inner = extract_inner(trimmed, "assert(");
            let (code, pred, _arity) = compile_fact(inner);
            let addr = db_code.len();
            db_code.extend(code);
            predicate_table.entry(pred).or_insert(Vec::new()).push(addr);
        } else if trimmed.starts_with("retract(") {
            let inner = extract_inner(trimmed, "retract(");
            let success = retract_clause(&mut db_code, &mut predicate_table, inner);
            println!("{}", if success { "true." } else { "false." });
        } else if trimmed.starts_with("?-") {
            // Treat a line starting with ?- as a query.
            query_code = Some(compile_query(trimmed));
        } else {
            // Otherwise, treat it as a fact clause.
            let (code, pred, _arity) = compile_fact(trimmed);
            let addr = db_code.len();
            db_code.extend(code);
            predicate_table.entry(pred).or_insert(Vec::new()).push(addr);
        }
    }

    if query_code.is_none() {
        println!("No query found in the program.");
        return;
    }
    let query_code = query_code.unwrap();
    let query_start = db_code.len();
    let mut full_code = db_code;
    full_code.extend(query_code);

    let mut machine = Machine::new(100, full_code);
    machine.pc = query_start;
    for (pred, addrs) in predicate_table.into_iter() {
        for addr in addrs {
            machine.register_predicate(pred.clone(), addr);
        }
    }
    machine.verbose = true;

    match machine.run() {
        Ok(_) => {
            println!("Query succeeded. Variable bindings:");
            for (var_id, name) in machine.variable_names.iter() {
                let binding = machine.uf.resolve(&Term::Var(*var_id));
                println!("  {} = {:?}", name, binding);
            }
        }
        Err(e) => {
            println!("Error during execution: {}", e);
        }
    }
}

/// Runs an interactive REPL.
/// Every command is entered at the `?-` prompt. Commands may be:
///   • A query (e.g., `?- parent(john, X).`)
///   • An assertion (e.g., `?- assert(parent(john, mary)).`)
///   • A retraction (e.g., `?- retract(parent(john, mary)).`)
///   • A directive (e.g., `?- :- dynamic(parent/2).`)
fn run_repl() {
    let mut db_code: Vec<Instruction> = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();

    println!("Welcome to LAM Prolog REPL.");
    println!("Enter queries (e.g., ?- parent(john, X).) and commands such as");
    println!("  assert(<clause>).    to add a clause,");
    println!("  retract(<clause>).   to remove a clause, and");
    println!("  :- dynamic(<pred>/<arity>).   to declare a predicate dynamic.");
    println!("Type 'halt.' or 'quit.' to exit.\n");

    loop {
        print!("?- ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("halt.") || trimmed.eq_ignore_ascii_case("quit.") {
            break;
        }
        // Remove any duplicate leading ?-.
        let cleaned = if trimmed.starts_with("?-") {
            trimmed.trim_start_matches("?-").trim()
        } else {
            trimmed
        };

        if cleaned.starts_with("assert(") {
            let inner = extract_inner(cleaned, "assert(");
            let (code, pred, _arity) = compile_fact(inner);
            let addr = db_code.len();
            db_code.extend(code);
            predicate_table.entry(pred).or_insert(Vec::new()).push(addr);
            println!("true.");
        } else if cleaned.starts_with("retract(") {
            let inner = extract_inner(cleaned, "retract(");
            if retract_clause(&mut db_code, &mut predicate_table, inner) {
                println!("true.");
            } else {
                println!("false.");
            }
        } else if cleaned.starts_with(":- dynamic(") {
            // Acknowledge dynamic declarations.
            println!("true.");
        } else {
            // Otherwise, treat the input as a query.
            let query_instr = compile_query(cleaned);
            let query_start = db_code.len();
            let mut full_code = db_code.clone();
            full_code.extend(query_instr);
            let mut machine = Machine::new(100, full_code);
            machine.pc = query_start;
            for (pred, addrs) in predicate_table.iter() {
                for &addr in addrs {
                    machine.register_predicate(pred.clone(), addr);
                }
            }
            machine.verbose = false;
            match machine.run() {
                Ok(_) => {
                    println!("Yes.");
                    for (var_id, name) in machine.variable_names.iter() {
                        let binding = machine.uf.resolve(&Term::Var(*var_id));
                        println!("  {} = {:?}", name, binding);
                    }
                }
                Err(e) => {
                    println!("No. ({})", e);
                }
            }
        }
    }
    println!("Goodbye.");
}

/// A helper function to extract the inner text from a command like "assert(...)." or "retract(...)."
fn extract_inner<'a>(input: &'a str, prefix: &str) -> &'a str {
    let without_prefix = input.trim_start_matches(prefix).trim();
    let without_trailing = if without_prefix.ends_with(").") {
        &without_prefix[..without_prefix.len() - 2]
    } else if without_prefix.ends_with(")") {
        &without_prefix[..without_prefix.len() - 1]
    } else {
        without_prefix
    };
    without_trailing.trim()
}

/// Retracts (removes) a fact clause from the database. For simplicity, this removes
/// the first clause registered for the predicate of the given clause text.
fn retract_clause(
    _db_code: &mut Vec<Instruction>,
    predicate_table: &mut HashMap<String, Vec<usize>>,
    clause: &str,
) -> bool {
    let (_code, pred, _arity) = compile_fact(clause);
    if let Some(clause_addrs) = predicate_table.get_mut(&pred) {
        if !clause_addrs.is_empty() {
            clause_addrs.remove(0);
            return true;
        }
    }
    false
}

/// Compiles a fact clause into LAM instructions using Get‑instructions (for unification).
/// The clause should be of the form:
///     predicate(arg1, arg2, ..., argN).
fn compile_fact(line: &str) -> (Vec<Instruction>, String, usize) {
    let line = line.trim().trim_end_matches('.').trim();
    let open_paren = line.find('(').expect("Expected '(' in fact");
    let close_paren = line.rfind(')').expect("Expected ')' in fact");
    let pred = line[..open_paren].trim().to_string();
    let args_str = &line[open_paren + 1..close_paren];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    let arity = args.len();

    let mut instructions = Vec::new();
    let mut var_map: HashMap<String, usize> = HashMap::new();
    let mut next_var_id = 1000; // Fact-local variable IDs

    // Use Get* instructions so that when a query is made, unification occurs.
    for (i, arg) in args.iter().enumerate() {
        let term = parse_term(arg, &mut var_map, &mut next_var_id);
        match term {
            Term::Const(n) => {
                instructions.push(Instruction::GetConst { register: i, value: n });
            }
            Term::Str(s) => {
                instructions.push(Instruction::GetStr { register: i, value: s });
            }
            Term::Var(var_id) => {
                instructions.push(Instruction::GetVar {
                    register: i,
                    var_id,
                    name: arg.to_string(),
                });
            }
            _ => {}
        }
    }
    let reg_list: Vec<usize> = (0..arity).collect();
    instructions.push(Instruction::BuildCompound {
        target: arity,
        functor: pred.clone(),
        arg_registers: reg_list,
    });
    instructions.push(Instruction::Proceed);
    (instructions, pred, arity)
}

/// Compiles a query into LAM instructions. The query should be of the form:
///     predicate(arg1, arg2, ..., argN).
fn compile_query(line: &str) -> Vec<Instruction> {
    let line = line.trim().trim_end_matches('.').trim();
    let open_paren = line.find('(').expect("Expected '(' in query");
    let close_paren = line.rfind(')').expect("Expected ')' in query");
    let pred = line[..open_paren].trim().to_string();
    let args_str = &line[open_paren + 1..close_paren];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    let arity = args.len();

    let mut instructions = Vec::new();
    let mut var_map: HashMap<String, usize> = HashMap::new();
    let mut next_var_id = 100; // Query variable IDs start at 100

    // For queries we use Put* instructions to build the goal.
    for (i, arg) in args.iter().enumerate() {
        let term = parse_term(arg, &mut var_map, &mut next_var_id);
        match term {
            Term::Const(n) => {
                instructions.push(Instruction::PutConst { register: i, value: n });
            }
            Term::Str(s) => {
                instructions.push(Instruction::PutStr { register: i, value: s });
            }
            Term::Var(var_id) => {
                instructions.push(Instruction::PutVar {
                    register: i,
                    var_id,
                    name: arg.to_string(),
                });
            }
            _ => {}
        }
    }
    let reg_list: Vec<usize> = (0..arity).collect();
    instructions.push(Instruction::BuildCompound {
        target: arity,
        functor: pred.clone(),
        arg_registers: reg_list,
    });
    instructions.push(Instruction::Call { predicate: pred });
    instructions.push(Instruction::Proceed);
    instructions.push(Instruction::Halt);
    instructions
}

/// A simple term parser that distinguishes integers, quoted strings, variables,
/// and atoms. Variables are tokens whose first character is uppercase.
fn parse_term(
    s: &str,
    var_map: &mut HashMap<String, usize>,
    next_var_id: &mut usize,
) -> Term {
    if let Ok(n) = s.parse::<i32>() {
        return Term::Const(n);
    }
    if s.starts_with('"') && s.ends_with('"') {
        return Term::Str(s.trim_matches('"').to_string());
    }
    if s.chars().next().unwrap().is_uppercase() {
        if let Some(&id) = var_map.get(s) {
            return Term::Var(id);
        } else {
            let id = *next_var_id;
            *next_var_id += 1;
            var_map.insert(s.to_string(), id);
            return Term::Var(id);
        }
    }
    // Otherwise, treat it as an atom.
    Term::Str(s.to_string())
}
