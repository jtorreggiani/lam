// tests/test_backtracking_variables.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for validating the LAM's trail mechanism for variable bindings during backtracking.
///
/// This test simulates the following Prolog behavior:
///
/// ```prolog
/// test_var_backtracking :-
///     X,                       % PutVar reg0, "X" (X is initially unbound)
///     (                        % Create a choice point (state with X unbound is saved)
///         X = 100,           % GetConst reg0, 100 (first alternative: binds X to 100)
///         fail               % Force failure, triggering backtracking (undoes the binding from step 2)
///         ;                  % Backtrack to the choice point
///         X = 300            % GetConst reg0, 300 (second alternative: binds X to 300)
///     ).
/// % Expected result:
/// %   X = 300.
/// ```
///
/// Step-by-step:
/// 1. Register 0 is set to variable X (unbound).
/// 2. A Choice instruction saves the state, including an empty trail (trail length 0).
/// 3. GetConst reg0, 100 unifies X with 100, pushing a trail entry. (Trail length becomes 1.)
/// 4. Fail is executed, causing backtracking:
///    - The trail is unwound back to the saved trail length (0).
///    - The saved substitution (with X unbound) is restored.
/// 5. Then, GetConst reg0, 300 unifies X with 300, pushing a new trail entry. (Trail length becomes 1.)
/// 6. Finally, the substitution binds X to 300.
/// 
/// We then verify that:
/// - The substitution environment binds "X" to Const(300).
/// - The trail length is 1 (since the final unification pushed one trail entry).
/// - The choice stack is empty.
#[test]
fn test_backtracking_variables() {
    let code = vec![
        // Step 0: PutVar reg0, "X" – set register 0 to the unbound variable X.
        Instruction::PutVar { register: 0, name: "X".to_string() },
        // Step 1: Create a choice point (the current trail length is 0).
        Instruction::Choice,
        // Step 2: First alternative: GetConst reg0, 100 – unify X with 100.
        Instruction::GetConst { register: 0, value: 100 },
        // Step 3: Fail – triggers backtracking (undoes the binding from step 2).
        Instruction::Fail,
        // Step 4: Second alternative: GetConst reg0, 300 – unify X with 300.
        Instruction::GetConst { register: 0, value: 300 },
    ];
    
    let mut machine = Machine::new(1, code);
    let _ = machine.run();
    
    // After execution, we expect that:
    // - The substitution binds "X" to Const(300).
    assert_eq!(machine.substitution.get("X"), Some(&Term::Const(300)));
    // - The trail length is 1 because the final unification (step 4) pushes one trail entry.
    assert_eq!(machine.trail.len(), 1);
    // - The choice stack is empty.
    assert_eq!(machine.choice_stack.len(), 0);
}
