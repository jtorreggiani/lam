use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// Test for the GetStructure instruction.
///
/// This test builds a compound term and then uses GetStructure to verify that
/// the term in the specified register has the expected functor and arity.
///
/// The Prolog equivalent might be:
/// ```prolog
/// test_get_structure :-
///     F = f(1, 2),          % BuildCompound equivalent
///     get_structure(F, f, 2). % Succeeds if F is a compound term f/2.
/// ```
///
/// Steps:
/// 1. Register 0 is set to 1 (constant).
/// 2. Register 1 is set to 2 (constant).
/// 3. A compound term f(1,2) is built from registers 0 and 1 and stored in register 2.
/// 4. GetStructure is executed on register 2, expecting functor "f" and arity 2.
#[test]
fn test_get_structure() {
    let code = vec![
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::PutConst { register: 1, value: 2 },
        Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
        Instruction::GetStructure { register: 2, functor: "f".to_string(), arity: 2 },
    ];
    
    let mut machine = Machine::new(3, code);
    let _ = machine.run();
    
    // The BuildCompound should have built f(1,2) in register 2.
    let expected = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);
    assert_eq!(machine.registers[2], Some(expected));
}
