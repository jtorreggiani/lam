#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_backtracking_constants() {
        let code = vec![
            // Instruction 0: Set reg0 to 10.
            Instruction::PutConst { register: 0, value: 10 },
            // Instruction 1: Create a choice point with alternative branch at address 4.
            Instruction::Choice { alternative: 4 },
            // Instruction 2: First alternative: set reg1 to 20.
            Instruction::PutConst { register: 1, value: 20 },
            // Instruction 3: Force failure.
            Instruction::Fail,
            // Instruction 4: Second alternative: set reg1 to 30.
            Instruction::PutConst { register: 1, value: 30 },
        ];
        
        let mut machine = Machine::new(2, code);
        machine.run().expect("Machine run should succeed");
        
        // Verify that reg0 remains 10 and reg1 becomes 30.
        assert_eq!(machine.registers[0], Some(Term::Const(10)));
        assert_eq!(machine.registers[1], Some(Term::Const(30)));
        // The choice stack should be empty.
        assert_eq!(machine.choice_stack.len(), 0);
    }
}
