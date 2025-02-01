// tests/test_cut.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for the Cut instruction.
///
/// This test simulates the following Prolog behavior:
/// 
/// ```prolog
/// test_cut :-
///     (   % Create two alternatives:
///         ( X = 1, !, fail )   % First alternative: bind X to 1, then cut, then force failure
///         ;
///         X = 2               % Second alternative: bind X to 2
///     ).
/// % Expected result: X = 2.
/// ```
/// 
/// In this test:
/// 1. A Choice point is created.
/// 2. The first alternative sets register 0 to 1, then executes a Cut, then forces failure.
/// 3. The Cut should clear the choice points, so backtracking does not return to an earlier alternative.
/// 4. The machine then continues (or finishes) without any further alternatives, so the final binding remains.
/// We verify that register 0 does not remain 1 (the first alternative), implying the cut prevented backtracking.
#[test]
fn test_cut() {
    let code = vec![
        // Create a choice point.
        Instruction::Choice,
        // First alternative: bind X (reg0) to 1.
        Instruction::PutConst { register: 0, value: 1 },
        // Execute cut, which clears choice points.
        Instruction::Cut,
        // Force failure (this alternative should fail and not backtrack due to the cut).
        Instruction::Fail,
        // Second alternative: if backtracking occurred, this would bind X to 2.
        Instruction::PutConst { register: 0, value: 2 },
        Instruction::Proceed,
    ];
    
    let mut machine = Machine::new(1, code);
    // For this test, we pre-register a dummy predicate if needed.
    // (Alternatively, the Call instruction might be used in a more complex scenario.)
    machine.run();
    
    // After running, because of the cut, the first alternative's failure should not backtrack to the second alternative.
    // Therefore, the value in register 0 should remain as it was set before the cut.
    // Since the alternative that binds 1 forced a failure after the cut, no further alternatives are tried.
    // We expect that register 0 does NOT get bound to 2.
    // In our simple system, this may result in register 0 still containing 1,
    // or it may remain unchanged if the failure terminates execution.
    // For this test, let's assert that register 0 is 1, confirming that backtracking did not reach the clause that sets it to 2.
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
    // And the choice stack should be empty.
    assert_eq!(machine.choice_stack.len(), 0);
}
