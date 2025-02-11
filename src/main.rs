// src/main.rs
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::error::Error;

// Import the Prolog compiler.
use lam::prolog::interpreter::compile_prolog;
// Import the LAM instruction parser.
use lam::machine::instruction_parser::parse_instructions;
// Import the LAM machine.
use lam::machine::core::Machine;

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve command-line arguments.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lam-tool <file> [--execute | -x]");
        std::process::exit(1);
    }
    let filename = &args[1];
    // Check for the optional execute flag.
    let execute_flag = args.iter().any(|arg| arg == "--execute" || arg == "-x");

    let path = Path::new(filename);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("lam") => {
            // If a .lam file is provided, parse its instructions and run it.
            let content = fs::read_to_string(filename)
                .unwrap_or_else(|e| panic!("Failed to read file '{}': {}", filename, e));
            let instructions = parse_instructions(&content)
                .unwrap_or_else(|e| panic!("Failed to parse LAM instructions: {}", e));
            println!("Executing LAM program from file '{}':", filename);
            let mut machine = Machine::new(10, instructions);
            machine.run().unwrap_or_else(|e| {
                eprintln!("Machine execution error: {:?}", e);
                std::process::exit(1);
            });
        },
        Some("pl") => {
            // For Prolog (.pl) files, compile them to LAM instructions.
            let program = fs::read_to_string(filename)
                .unwrap_or_else(|e| panic!("Failed to read file '{}': {}", filename, e));
            let (instructions, pred_table) = compile_prolog(&program)
                .unwrap_or_else(|e| panic!("Failed to compile Prolog program: {}", e));

            // For debugging, print the predicate table.
            println!("Predicate Table:");
            for (pred, addrs) in pred_table.iter() {
                println!("  {} -> {:?}", pred, addrs);
            }

            if execute_flag {
                // If the execute flag is provided, execute the compiled LAM program.
                println!("Executing compiled LAM program:");
                let mut machine = Machine::new(10, instructions);
                machine.run().unwrap_or_else(|e| {
                    eprintln!("Machine execution error: {:?}", e);
                    std::process::exit(1);
                });
            } else {
                // Otherwise, write the compiled instructions to a new file with .lam extension.
                let mut output = String::new();
                for instr in instructions.iter() {
                    output.push_str(&format!("{}\n", instr));
                }
                let output_path = path.with_extension("lam");
                let mut file = fs::File::create(&output_path)
                    .unwrap_or_else(|e| panic!("Failed to create output file '{:?}': {}", output_path, e));
                file.write_all(output.as_bytes())
                    .unwrap_or_else(|e| panic!("Failed to write output file '{:?}': {}", output_path, e));
                println!("Compiled code written to {:?}", output_path);
            }
        },
        _ => {
            eprintln!("Unsupported file extension. Please provide a .lam or .pl file.");
            std::process::exit(1);
        }
    }

    Ok(())
}
