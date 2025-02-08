// tests/test_cut.rs

use lam::machine::instruction::Instruction;
use lam::machine::core::Machine;
use lam::machine::term::Term;

// Test for the Cut instruction.
//
// This test simulates the following Prolog behavior:
// 
//   test_cut :-
//       (   % Create two alternatives:
//           ( X = 1, !, fail )   % First alternative: bind X to 1, cut, then force failure
//           ;
//           X = 2               % Second alternative: bind X to 2
//       ).
// Expected result: X = 1 (since the cut prevents backtracking to the second alternative).
//
// Additionally, we verify that after the cut, any remaining choice point has a call level less
// than the current call level.
#[test]
fn test_cut() {
    let code = vec![
        // Create a choice point; record alternative branch at address 4.
        Instruction::Choice { alternative: 4 },
        // First alternative: bind X (reg0) to 1.
        Instruction::PutConst { register: 0, value: 1 },
        // Execute cut; should remove all choice points created in this call.
        Instruction::Cut,
        // Force failure (this alternative will fail and should not backtrack)
        Instruction::Fail,
        // Second alternative: if backtracking occurred, this would bind X to 2.
        Instruction::PutConst { register: 0, value: 2 },
        Instruction::Proceed,
    ];
    
    let mut machine = Machine::new(1, code);
    let _ = machine.run();
    
    // Verify that register 0 remains 1 (the binding before the cut)
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
    // Verify that the choice stack is empty (or that any remaining choice points are from an outer call)
    assert!(machine.choice_stack.iter().all(|cp| cp.call_level < machine.control_stack.len()));
}
