// tests/test_backtracking_constants.rs

use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
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
    let code = vec![
        // Instruction 0: Set reg0 to 10.
        Instruction::PutConst { register: 0, value: 10 },
        // Instruction 1: Create a choice point; record that the alternative branch is at address 4.
        Instruction::Choice { alternative: 4 },
        // Instruction 2: First alternative: set reg1 to 20.
        Instruction::PutConst { register: 1, value: 20 },
        // Instruction 3: Force failure.
        Instruction::Fail,
        // Instruction 4: Second alternative: set reg1 to 30.
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
