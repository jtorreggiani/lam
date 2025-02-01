// tests/test_backtracking.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for validating the LAM's backtracking mechanism through choice points and failure.
///
/// Equivalent Prolog code:
/// ```prolog
/// test_choice_and_fail :-
///     X = 10,                   % PutConst reg0, 10
///     (                         % Choice
///         Y = 20,              % PutConst reg1, 20
///         fail                 % Fail
///         ;                    % Backtrack to choice point
///         Y = 30              % PutConst reg1, 30
///     ).
///
/// % Expected query result:
/// % ?- test_choice_and_fail.
/// % X = 10,
/// % Y = 30.
/// ```
///
/// The test executes the following sequence:
/// 1. Sets register 0 to 10 (persists through backtracking)
/// 2. Creates a choice point saving machine state
/// 3. First alternative: sets register 1 to 20
/// 4. Forces failure and triggers backtracking
/// 5. Second alternative: sets register 1 to 30
///
/// LAM Instructions:
/// - PutConst: Stores constant values in registers
/// - Choice: Creates backtracking point by saving machine state
/// - Fail: Forces backtracking to most recent choice point
///
/// The test verifies:
/// - Values set before choice points remain unchanged
/// - Backtracking successfully executes alternative paths
/// - Choice stack is properly cleaned up after execution
#[test]
fn test_choice_and_fail() {
    // Program structure:
    // 0: PutConst   reg0, 10     // Store initial value that should persist
    // 1: Choice                  // Save state for backtracking
    // 2: PutConst   reg1, 20     // First alternative (will be undone)
    // 3: Fail                    // Trigger backtracking
    // 4: PutConst   reg1, 30     // Second alternative (final state)
    let code = vec![
        // Store 10 in reg0 - this value should persist through backtracking
        Instruction::PutConst { register: 0, value: 10 },
        // Create choice point saving current machine state
        Instruction::Choice,
        // First alternative - will be undone by backtracking
        Instruction::PutConst { register: 1, value: 20 },
        // Force backtracking to the choice point
        Instruction::Fail,
        // Second alternative - executed after backtracking
        Instruction::PutConst { register: 1, value: 30 },
    ];
    
    let mut machine = Machine::new(2, code);
    machine.run();
    
    // Verify test conditions:
    // 1. reg0 should still contain 10 (pre-choice point value persists)
    assert_eq!(machine.registers[0], Some(Term::Const(10)));
    // 2. reg1 should contain 30 (second alternative after backtracking)
    assert_eq!(machine.registers[1], Some(Term::Const(30)));
    // 3. Choice stack should be empty (cleanup after backtracking)
    assert_eq!(machine.choice_stack.len(), 0);
}
