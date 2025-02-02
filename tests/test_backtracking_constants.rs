// tests/test_backtracking_constants.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for validating the LAM's backtracking mechanism for constant values.
///
/// This test simulates the following Prolog behavior:
/// 
/// ```prolog
/// test_const_backtracking :-
///     X = 10,                % PutConst reg0, 10
///     (                      % Create a choice point
///         Y = 20,            % PutConst reg1, 20 (first alternative)
///         fail               % Force failure, triggering backtracking
///         ;                  % Backtrack to choice point
///         Y = 30             % PutConst reg1, 30 (second alternative)
///     ).
/// 
/// % Expected result:
/// %   X = 10,
/// %   Y = 30.
/// ```
/// 
/// In this test:
/// 1. Register 0 is set to 10 (this value persists through backtracking).
/// 2. A choice point is created to save the current state.
/// 3. The first alternative sets register 1 to 20 (which is later undone).
/// 4. A Fail instruction forces backtracking, restoring the saved state.
/// 5. The second alternative sets register 1 to 30.
/// 6. Finally, the test verifies that:
///    - reg0 still contains 10,
///    - reg1 contains 30,
///    - and the choice stack is empty.
#[test]
fn test_backtracking_constants() {
    // Program structure:
    // 0: PutConst   reg0, 10     // Store 10 in reg0 (persists)
    // 1: Choice                  // Save state for backtracking
    // 2: PutConst   reg1, 20     // First alternative: reg1 = 20 (will be undone)
    // 3: Fail                    // Trigger backtracking
    // 4: PutConst   reg1, 30     // Second alternative: reg1 = 30
    let code = vec![
        Instruction::PutConst { register: 0, value: 10 },
        Instruction::Choice,
        Instruction::PutConst { register: 1, value: 20 },
        Instruction::Fail,
        Instruction::PutConst { register: 1, value: 30 },
    ];
    
    let mut machine = Machine::new(2, code);
    let _ = machine.run();
    
    // Verify:
    // - reg0 should remain 10.
    // - reg1 should be 30 (the alternative after backtracking).
    // - The choice stack should be empty.
    assert_eq!(machine.registers[0], Some(Term::Const(10)));
    assert_eq!(machine.registers[1], Some(Term::Const(30)));
    assert_eq!(machine.choice_stack.len(), 0);
}
