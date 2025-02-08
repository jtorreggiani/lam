#[cfg(test)]
mod tests {
    use lam::machine::error_handling::MachineError;
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;

    #[test]
    fn test_register_out_of_bounds_error() {
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
        let code = vec![
            Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
        ];
        let mut machine = Machine::new(1, code);
        let result = machine.run();
        match result {
            Err(MachineError::UninitializedRegister(n)) => assert_eq!(n, 0),
            _ => panic!("Expected UninitializedRegister error for IndexedCall"),
        }
    }
}
