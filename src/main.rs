//! Entry point for the LAM interpreter.
//! This module reads a program file, parses it, and executes it on the abstract machine.

use std::env;
use std::fs;
use lam::machine::Machine;
use lam::parser::parse_program;

fn main() {
    // Initialize the logger (env_logger reads log level from RUST_LOG).
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
