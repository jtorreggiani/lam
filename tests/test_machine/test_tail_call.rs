// tests/test_tail_call.rs

use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
use lam::machine::term::Term;

/// Test for tail-call optimization.
///
/// The program simulates a tail call as follows:
/// - The main program allocates an environment and sets a local variable.
/// - A TailCall instruction calls predicate "p".
///   The tail call should first deallocate the current environment frame.
/// - Predicate "p" (starting at address 4) simply puts 200 into register 0 and then proceeds.
/// - The expected result is that register 0 contains 200 and the environment stack is empty.
#[test]
fn test_tail_call() {
    // Main program:
    let code = vec![
        // Allocate an environment frame.
        Instruction::Allocate { n: 1 },
        // Set local variable 0 to 100.
        Instruction::SetLocal { index: 0, value: Term::Const(100) },
        // Tail call to predicate "p".
        Instruction::TailCall { predicate: "p".to_string() },
        // Dummy instruction (should not execute if tail call works):
        Instruction::PutConst { register: 0, value: 999 },
        // Predicate "p" code (starts at index 4):
        Instruction::PutConst { register: 0, value: 200 },
        // Return from predicate "p".
        Instruction::Proceed,
    ];
    let mut machine = Machine::new(1, code);
    
    // Register predicate "p" to start at index 4.
    machine.register_predicate("p".to_string(), 4);
    
    let _ = machine.run();
    
    // Verify that register 0 was set to 200 by predicate "p".
    assert_eq!(machine.registers[0], Some(Term::Const(200)));
    // Verify that the environment stack is empty (tail call deallocated it).
    assert_eq!(machine.environment_stack.len(), 0);
}
