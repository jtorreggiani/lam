use lam::machine::{Machine, Instruction, MachineError};

#[test]
fn test_register_out_of_bounds_error() {
    // Attempt to write to register 5 while only 3 registers are available.
    let code = vec![Instruction::PutConst { register: 5, value: 10 }];
    let mut machine = Machine::new(3, code);
    let result = machine.run();
    match result {
        Err(MachineError::RegisterOutOfBounds(n)) => assert_eq!(n, 5),
        _ => panic!("Expected RegisterOutOfBounds error with register 5"),
    }
}

#[test]
fn test_uninitialized_register_error() {
    // Attempt to perform a GetConst on an uninitialized register 0.
    let code = vec![Instruction::GetConst { register: 0, value: 42 }];
    let mut machine = Machine::new(1, code);
    let result = machine.run();
    match result {
        Err(MachineError::UninitializedRegister(n)) => assert_eq!(n, 0),
        _ => panic!("Expected UninitializedRegister error on register 0"),
    }
}

#[test]
fn test_unification_failure_error() {
    // Set register 0 to 42 and then try to unify it with 100.
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 100 },
    ];
    let mut machine = Machine::new(1, code);
    let result = machine.run();
    match result {
        Err(MachineError::UnificationFailed(msg)) => {
            assert!(msg.contains("Cannot unify"), "Unexpected error message: {}", msg);
        }
        _ => panic!("Expected UnificationFailed error"),
    }
}

#[test]
fn test_deallocate_without_environment_error() {
    // Attempt to deallocate an environment when none has been allocated.
    let code = vec![Instruction::Deallocate];
    let mut machine = Machine::new(1, code);
    let result = machine.run();
    match result {
        Err(MachineError::EnvironmentMissing) => {},
        _ => panic!("Expected EnvironmentMissing error"),
    }
}

#[test]
fn test_get_structure_on_constant_error() {
    // Put a constant in register 0 and then try to get its structure.
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetStructure { register: 0, functor: "f".to_string(), arity: 2 },
    ];
    let mut machine = Machine::new(1, code);
    let result = machine.run();
    match result {
        Err(MachineError::NotACompoundTerm(reg)) => assert_eq!(reg, 0),
        _ => panic!("Expected NotACompoundTerm error on register 0"),
    }
}

#[test]
fn test_indexed_call_uninitialized_register_error() {
    // Attempt an IndexedCall using register 0, which is uninitialized.
    let code = vec![
        Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
    ];
    let mut machine = Machine::new(1, code);
    let result = machine.run();
    match result {
        Err(MachineError::UninitializedRegister(n)) => assert_eq!(n, 0),
        _ => panic!("Expected UninitializedRegister error on register 0 for IndexedCall"),
    }
}
