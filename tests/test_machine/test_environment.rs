// tests/test_environment.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test environment allocation, local variable assignment, and retrieval.
///
/// This program simulates:
/// 1. Allocating an environment frame of size 2.
/// 2. Setting local variable 0 to constant 42.
/// 3. Setting local variable 1 to constant 99.
/// 4. Retrieving local variable 0 into register 0.
/// 5. Retrieving local variable 1 into register 1.
/// 6. Deallocating the environment frame.
#[test]
fn test_environment() {
    let code = vec![
        // Allocate an environment with 2 local variables.
        Instruction::Allocate { n: 2 },
        // Set local variable 0 to 42.
        Instruction::SetLocal { index: 0, value: Term::Const(42) },
        // Set local variable 1 to 99.
        Instruction::SetLocal { index: 1, value: Term::Const(99) },
        // Retrieve local variable 0 into register 0.
        Instruction::GetLocal { index: 0, register: 0 },
        // Retrieve local variable 1 into register 1.
        Instruction::GetLocal { index: 1, register: 1 },
        // Deallocate the environment.
        Instruction::Deallocate,
    ];
    
    let mut machine = Machine::new(2, code);
    let _ = machine.run();
    
    // Verify that the registers hold the expected local values.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
    assert_eq!(machine.registers[1], Some(Term::Const(99)));
    // Verify that the environment stack is empty.
    assert_eq!(machine.environment_stack.len(), 0);
}
