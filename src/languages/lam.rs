// Combined Rust source code for the LAM interpreter.
// File: src/languages/lam.rs

//! Parser for textual LAM programs.
//!
//! Converts source code into a vector of Instructions.

use lam::machine::instruction::Instruction;
use lam::machine::arithmetic::parse_expression;
use lam::machine::term::Term;
use lam::machine::core::Machine;
use std::env;
use std::fs;

/// Splits the input line into tokens while keeping quoted strings together.
/// For example, the line:
///
///     PutStr 0 "Hello world"
///
/// will be tokenized as:
///
///     ["PutStr", "0", "\"Hello world\""]
fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in line.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push(c); // Keep the quotes so we can trim them later.
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Parses a LAM program (given as a string) into a vector of Instructions.
pub fn parse_program(input: &str) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    for (line_no, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Use the new tokenizer instead of simple whitespace splitting.
        let tokens = tokenize_line(line);
        if tokens.is_empty() {
            continue;
        }
        // Process the instruction based on the first token.
        match tokens[0].as_str() {
            "PutStr" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: PutStr expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].trim_matches('"').to_string();
                instructions.push(Instruction::PutStr { register, value });
            }
            "GetStr" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: GetStr expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].trim_matches('"').to_string();
                instructions.push(Instruction::GetStr { register, value });
            }
            "PutConst" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: PutConst expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].parse::<i32>()
                    .map_err(|_| format!("Line {}: invalid value", line_no + 1))?;
                instructions.push(Instruction::PutConst { register, value });
            }
            "PutVar" => {
                if tokens.len() != 4 {
                    return Err(format!("Line {}: PutVar expects 3 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let var_id = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid var_id", line_no + 1))?;
                let name = tokens[3].trim_matches('"').to_string();
                instructions.push(Instruction::PutVar { register, var_id, name });
            }
            "GetConst" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: GetConst expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].parse::<i32>()
                    .map_err(|_| format!("Line {}: invalid value", line_no + 1))?;
                instructions.push(Instruction::GetConst { register, value });
            }
            "GetVar" => {
                if tokens.len() != 4 {
                    return Err(format!("Line {}: GetVar expects 3 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let var_id = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid var_id", line_no + 1))?;
                let name = tokens[3].trim_matches('"').to_string();
                instructions.push(Instruction::GetVar { register, var_id, name });
            }
            "Call" => {
                // Special case: if the instruction is "Call write <string>",
                // then generate two instructions (PutStr followed by Call).
                if tokens[1].as_str() == "write" && tokens.len() == 3 {
                    let value = tokens[2].trim_matches('"').to_string();
                    instructions.push(Instruction::PutStr { register: 0, value });
                    instructions.push(Instruction::Call { predicate: "write".to_string() });
                } else {
                    if tokens.len() != 2 {
                        return Err(format!("Line {}: Call expects 1 argument", line_no + 1));
                    }
                    let predicate = tokens[1].to_string();
                    instructions.push(Instruction::Call { predicate });
                }
            }
            "Proceed" => {
                instructions.push(Instruction::Proceed);
            }
            "Choice" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: Choice expects 1 argument", line_no + 1));
                }
                let alternative = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid alternative", line_no + 1))?;
                instructions.push(Instruction::Choice { alternative });
            }
            "Allocate" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: Allocate expects 1 argument", line_no + 1));
                }
                let n = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid size", line_no + 1))?;
                instructions.push(Instruction::Allocate { n });
            }
            "Deallocate" => {
                instructions.push(Instruction::Deallocate);
            }
            "Fail" => {
                instructions.push(Instruction::Fail);
            }
            "ArithmeticIs" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: ArithmeticIs expects at least 2 arguments", line_no + 1));
                }
                let target = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid target", line_no + 1))?;
                let expr_str = tokens[2..].join(" ");
                let expression = parse_expression(&expr_str)
                    .map_err(|e| format!("Line {}: ArithmeticIs expression error: {}", line_no + 1, e))?;
                instructions.push(Instruction::ArithmeticIs { target, expression });
            }
            "SetLocal" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: SetLocal expects 2 arguments", line_no + 1));
                }
                let index = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid index", line_no + 1))?;
                let value = tokens[2].parse::<i32>()
                    .map_err(|_| format!("Line {}: invalid value", line_no + 1))?;
                instructions.push(Instruction::SetLocal { index, value: Term::Const(value) });
            }
            "GetLocal" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: GetLocal expects 2 arguments", line_no + 1));
                }
                let index = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid index", line_no + 1))?;
                let register = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                instructions.push(Instruction::GetLocal { index, register });
            }
            "GetStructure" => {
                if tokens.len() != 4 {
                    return Err(format!("Line {}: GetStructure expects 3 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let functor = tokens[2].to_string();
                let arity = tokens[3].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid arity", line_no + 1))?;
                instructions.push(Instruction::GetStructure { register, functor, arity });
            }
            "IndexedCall" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: IndexedCall expects 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let index_register = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid index register", line_no + 1))?;
                instructions.push(Instruction::IndexedCall { predicate, index_register });
            }
            "MultiIndexedCall" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: MultiIndexedCall expects at least 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let mut index_registers = Vec::new();
                for token in tokens.iter().skip(2) {
                    let reg = token.parse::<usize>()
                        .map_err(|_| format!("Line {}: invalid index register", line_no + 1))?;
                    index_registers.push(reg);
                }
                instructions.push(Instruction::MultiIndexedCall { predicate, index_registers });
            }
            "TailCall" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: TailCall expects 1 argument", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                instructions.push(Instruction::TailCall { predicate });
            }
            "AssertClause" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: AssertClause expects 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let address = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid address", line_no + 1))?;
                instructions.push(Instruction::AssertClause { predicate, address });
            }
            "RetractClause" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: RetractClause expects 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let address = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid address", line_no + 1))?;
                instructions.push(Instruction::RetractClause { predicate, address });
            }
            "Cut" => {
                instructions.push(Instruction::Cut);
            }
            "BuildCompound" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: BuildCompound expects at least 2 arguments", line_no + 1));
                }
                let target = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid target", line_no + 1))?;
                let functor = tokens[2].to_string();
                let mut arg_registers = Vec::new();
                for token in tokens.iter().skip(3) {
                    let reg = token.parse::<usize>()
                        .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                    arg_registers.push(reg);
                }
                instructions.push(Instruction::BuildCompound { target, functor, arg_registers });
            }
            "Halt" => {
                instructions.push(Instruction::Halt);
            }
            other => {
                return Err(format!("Line {}: Unknown instruction '{}'", line_no + 1, other));
            }
        }
    }
    Ok(instructions)
}

fn main() {
    // Initialize the logger (env_logger reads the log level from RUST_LOG).
    env_logger::init();
    // Expect the program filename as the first command-line argument.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run <program.lam>");
        return;
    }
    let filename = &args[1];
    let program_str = fs::read_to_string(filename)
        .expect(&format!("Failed to read file: {}", filename));
    // Parse the program text into a vector of LAM instructions.
    let instructions = parse_program(&program_str)
        .expect("Failed to parse program");
    // Create the machine (with a generous register count) and enable verbose logging.
    let mut machine = Machine::new(100, instructions);
    machine.verbose = true;
    // Run the machine and report any errors.
    if let Err(e) = machine.run() {
        eprintln!("Error during execution: {}", e);
    }
}
