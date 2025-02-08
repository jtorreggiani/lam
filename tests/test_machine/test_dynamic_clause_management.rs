#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;

    #[test]
    fn test_dynamic_clause_management() {
        // Program:
        // 0: AssertClause for predicate "p" at address 3.
        // 1: Call "p" (jumps to address 3).
        // 2: Proceed after call.
        // 3: PutConst reg0, 1 (clause body sets reg0 to 1).
        // 4: Proceed.
        // 5: RetractClause for predicate "p" at address 3.
        // 6: Call "p" again (this should fail because there is no clause).
        // 7: Proceed.
        // 8: Halt.
        let code = vec![
            Instruction::AssertClause { predicate: "p".to_string(), address: 3 },
            Instruction::Call { predicate: "p".to_string() },
            Instruction::Proceed,
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::Proceed,
            Instruction::RetractClause { predicate: "p".to_string(), address: 3 },
            Instruction::Call { predicate: "p".to_string() },
            Instruction::Proceed,
            Instruction::Halt,
        ];

        let mut machine = Machine::new(1, code);
        let result = machine.run();
        match result {
            Err(MachineError::PredicateClauseNotFound(_))
            | Err(MachineError::PredicateNotFound(_)) => {
                // Expected error: after retraction, there is no clause for predicate "p".
            }
            other => panic!("Expected predicate not found error, got {:?}", other),
        }
        // Verify that register 0 remains as set by the first call.
        assert_eq!(machine.registers[0], Some(Term::Const(1)));
    }
}
