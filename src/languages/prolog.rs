//! A SWI‑Prolog–style interpreter built on top of the LAM abstract machine.
//!
//! In file mode (when a file argument is provided), the interpreter scans for an
//! initialization directive (e.g., :- initialization(main).) and executes that goal
//! programmatically—printing only the output produced by built‑ins.
//!
//! In REPL mode (no file argument), an interactive prompt is shown.

use std::collections::HashMap;
use std::io::{self, Write};

use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
use lam::machine::term::Term;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // File mode: run program from file.
        let filename = &args[1];
        let content = std::fs::read_to_string(filename)
            .expect("Failed to read the source file");
        run_prolog_program(&content);
    } else {
        // REPL mode: start interactive prompt.
        run_repl();
    }
}

/// In file mode the interpreter compiles the file, looks for an initialization
/// directive (e.g., :- initialization(main).), compiles that as the query, and runs it.
fn run_prolog_program(source: &str) {
    let mut db_code: Vec<Instruction> = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();
    let mut query_code: Option<Vec<Instruction>> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with(":- initialization(") {
            // Extract and compile the initialization goal.
            let inner = extract_inner(trimmed, ":- initialization(");
            query_code = Some(compile_query(inner));
        } else if trimmed.starts_with("assert(") {
            // Handle an assert command.
            let inner = extract_inner(trimmed, "assert(");
            let (code, pred, _arity) = compile_fact(inner);
            let addr = db_code.len();
            db_code.extend(code);
            predicate_table.entry(pred).or_insert(Vec::new()).push(addr);
        } else if trimmed.starts_with("retract(") {
            let inner = extract_inner(trimmed, "retract(");
            let success = retract_clause(&mut db_code, &mut predicate_table, inner);
            println!("{}", if success { "true." } else { "false." });
        } else {
            // Otherwise, treat it as a fact clause.
            let (code, pred, _arity) = compile_fact(trimmed);
            let addr = db_code.len();
            db_code.extend(code);
            predicate_table.entry(pred).or_insert(Vec::new()).push(addr);
        }
    }

    if query_code.is_none() {
        println!("No initialization query found in the program.");
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
    machine.verbose = false; // Suppress extra diagnostic output.
    match machine.run() {
        Ok(_) => {
            // In file mode, the built-in predicates produce the desired output.
        }
        Err(e) => {
            println!("Error during execution: {}", e);
        }
    }
}

/// The REPL prints a ?- prompt and reads commands interactively.
fn run_repl() {
    let mut db_code: Vec<Instruction> = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();

    println!("Welcome to LAM Prolog REPL.");
    println!("Enter queries (e.g., ?- hello(X).) and commands such as:");
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
            // For now, just acknowledge dynamic declarations.
            println!("true.");
        } else if cleaned.starts_with(":- initialization(") {
            let inner = extract_inner(cleaned, ":- initialization(");
            let query_instr = compile_query(inner);
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
                }
                Err(e) => {
                    println!("No. ({})", e);
                }
            }
        } else {
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

/// Helper: extracts the inner text from commands like "assert(...)." or ":- initialization(...)."
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

/// For simplicity, retract_clause removes the first clause registered for the predicate
/// named in the given clause text.
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

/// Compiles a fact clause (of the form: predicate(arg1, arg2, ..., argN).)
/// into a sequence of LAM instructions using Get‑instructions (for unification).
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
    let mut next_var_id = 1000;

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

/// Compiles a query (of the form: predicate(arg1, arg2, ..., argN).)
/// into a sequence of LAM instructions using Put‑instructions to build the goal.
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
    let mut next_var_id = 100;

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

/// A simple term parser that distinguishes integers, quoted strings, variables (tokens
/// beginning with an uppercase letter), and atoms.
fn parse_term(s: &str, var_map: &mut HashMap<String, usize>, next_var_id: &mut usize) -> Term {
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
