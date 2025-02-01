// tests/test_backtracking_variables.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for validating the LAM's trail mechanism for variable bindings during backtracking.
///
/// This test simulates the following Prolog behavior:
///
/// ```prolog
/// test_var_backtracking :-
///     X,                     % PutVar reg0, "X"
///     (                      % Create a choice point (state with X unbound)
///         X = 100,           % GetConst reg0, 100 (first alternative binding)
///         fail               % Force failure, triggering backtracking (undo binding)
///         ;                  % Backtrack to choice point
///         X = 300            % GetConst reg0, 300 (second alternative binding)
///     ).
/// 
/// % Expected result:
/// %   X = 300.
/// ```
/// 
/// In this test:
/// 1. Register 0 is initialized with an unbound variable "X".
/// 2. A choice point is created, saving the state (with X unbound).
/// 3. The first alternative attempts to bind X to 100 (recorded on the trail).
/// 4. A Fail instruction forces backtracking, undoing the binding made after the choice point.
/// 5. The second alternative then binds X to 300.
/// 6. The test verifies that:
///    - In the substitution environment, "X" is bound to Const(300),
///    - And both the trail and choice stacks are empty.
#[test]
fn test_backtracking_variables() {
    // Program structure:
    // 0: PutVar   reg0, "X"       // Set reg0 to variable X (unbound)
    // 1: Choice                   // Create a choice point (save state with X unbound)
    // 2: GetConst reg0, 100       // First alternative: attempt to bind X to 100
    // 3: Fail                     // Trigger backtracking (undo binding from step 2)
    // 4: GetConst reg0, 300       // Second alternative: bind X to 300
    let code = vec![
        Instruction::PutVar { register: 0, name: "X".to_string() },
        Instruction::Choice,
        Instruction::GetConst { register: 0, value: 100 },
        Instruction::Fail,
        Instruction::GetConst { register: 0, value: 300 },
    ];
    
    let mut machine = Machine::new(1, code);
    machine.run();
    
    // Verify:
    // - The substitution environment should have "X" bound to Const(300).
    //   (Note: The register may still show Var("X"), so we check the substitution.)
    assert_eq!(machine.substitution.get("X"), Some(&Term::Const(300)));
    // - The trail and choice stacks should be empty after backtracking.
    assert_eq!(machine.trail.len(), 0);
    assert_eq!(machine.choice_stack.len(), 0);
}
