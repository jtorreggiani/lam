// tests/test_machine.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

#[test]
fn test_put_const_instruction() {
    // Prolog equivalent:
    // ?- X = 42.
    // X = 42.
    
    let code = vec![Instruction::PutConst { register: 0, value: 42 }];
    let mut machine = Machine::new(2, code);

    // Before running, both registers should be None.
    assert_eq!(machine.registers[0], None);
    assert_eq!(machine.registers[1], None);

    // Execute one instruction.
    let cont = machine.step();
    assert!(cont);

    // After execution, register 0 should contain the constant 42.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
    // Register 1 remains unchanged.
    assert_eq!(machine.registers[1], None);

    // There should be no further instruction.
    assert!(!machine.step());
}

#[test]
fn test_put_var_instruction() {
    // Prolog equivalent:
    // ?- X = X.
    // true.
    
    let code = vec![Instruction::PutVar { register: 1, name: "X".to_string() }];
    let mut machine = Machine::new(2, code);

    // Run the machine.
    machine.run();

    // Register 1 should now contain a variable "X".
    assert_eq!(machine.registers[1], Some(Term::Var("X".to_string())));
}

#[test]
fn test_get_const_unification_success() {
    // Prolog equivalent:
    // ?- X = 42, X = 42.
    // X = 42.
    
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 42 },
    ];
    let mut machine = Machine::new(1, code);

    machine.run();

    // Unification should succeed and register 0 remains as 42.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
}

#[test]
fn test_get_const_unification_failure() {
    // Prolog equivalent:
    // ?- X = 42, X = 100.
    // false.
    
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 100 },
    ];
    let mut machine = Machine::new(1, code);

    machine.run();

    // Our current implementation simply prints an error on unification failure,
    // and leaves the register unchanged.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
}

#[test]
fn test_get_var_unification() {
    // Prolog equivalent:
    // ?- X = X, Y = X.
    // X = X,
    // Y = X.
    
    let code = vec![
        Instruction::PutVar { register: 0, name: "X".to_string() },
        Instruction::GetVar { register: 0, name: "Y".to_string() },
    ];
    let mut machine = Machine::new(1, code);

    machine.run();

    // The register should remain unchanged (still holds variable "X").
    assert_eq!(machine.registers[0], Some(Term::Var("X".to_string())));

    // The substitution environment should record that "Y" is bound to "X".
    let binding = machine.substitution.get("Y").cloned();
    assert_eq!(binding, Some(Term::Var("X".to_string())));
}