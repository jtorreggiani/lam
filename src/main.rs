use std::env;
use std::fs;
use lam::machine::core::Machine;
use lam::machine::instruction_parser::parse_instructions;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let contents = fs::read_to_string(filename)
            .unwrap_or_else(|e| panic!("Failed to read file '{}': {}", filename, e));
        let instructions = parse_instructions(&contents)
            .unwrap_or_else(|e| panic!("Failed to parse instructions: {}", e));
        // Create a machine with (say) 10 registers.
        let mut machine = Machine::new(10, instructions);
        machine.run().unwrap_or_else(|e| {
            eprintln!("Machine execution error: {:?}", e);
            std::process::exit(1);
        });
    } else {
        println!("Usage: lam <file.lam>");
        println!("Example: lam examples/hello.lam");
    }
}
