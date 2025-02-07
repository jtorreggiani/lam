// tests/test_build_compound.rs

use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
use lam::term::Term;

/// Test for building compound terms dynamically from register values.
///
/// The test simulates the following Prolog behavior:
/// 
/// ```prolog
/// test_build_compound :-
///     A = 42,                % PutConst reg0, 42
///     B = 99,                % PutConst reg1, 99
///     F = f(A, B),           % BuildCompound in reg2 using functor 'f'
///     % Expected: F = f(42, 99)
///     true.
/// ```
/// 
/// The sequence is as follows:
/// 1. Register 0 is set to 42.
/// 2. Register 1 is set to 99.
/// 3. The BuildCompound instruction constructs f(42, 99) from registers 0 and 1
///    and stores the result in register 2.
/// The test verifies that register 2 contains the compound term f(42, [99]).
#[test]
fn test_build_compound() {
    // Program structure:
    // 0: PutConst   reg0, 42
    // 1: PutConst   reg1, 99
    // 2: BuildCompound target=2, functor "f", arguments from registers [0, 1]
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::PutConst { register: 1, value: 99 },
        Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
    ];
    
    let mut machine = Machine::new(3, code);
    let _ = machine.run();
    
    // Expected compound term: f(42, 99)
    let expected = Term::Compound("f".to_string(), vec![Term::Const(42), Term::Const(99)]);
    
    assert_eq!(machine.registers[2], Some(expected));
}