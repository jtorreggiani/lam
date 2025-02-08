#[cfg(test)]
mod tests {
    use lam::machine::instruction::Instruction;
    use lam::machine::core::Machine;
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;

    #[test]
    fn test_cut() {
        let code = vec![
            // Create a choice point with an alternative at address 4.
            Instruction::Choice { alternative: 4 },
            // First alternative: bind reg0 to 1.
            Instruction::PutConst { register: 0, value: 1 },
            // Execute Cut to prune any choice points created in this call.
            Instruction::Cut,
            // Force failure: this will try to backtrack, but since the choice point was pruned,
            // we expect a NoChoicePoint error.
            Instruction::Fail,
            // Second alternative (should not be reached): bind reg0 to 2.
            Instruction::PutConst { register: 0, value: 2 },
            Instruction::Proceed,
        ];

        let mut machine = Machine::new(1, code);
        let result = machine.run();
        match result {
            Err(MachineError::NoChoicePoint) => {
                // Expected error: after the cut, the failure causes no choice point to be available.
            }
            other => panic!("Expected NoChoicePoint error, got {:?}", other),
        }
        // Verify that register 0 holds the value from the first alternative.
        assert_eq!(machine.registers[0], Some(Term::Const(1)));
    }
}
