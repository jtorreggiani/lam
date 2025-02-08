#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::arithmetic::Expression;
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;

    /// Test that the ArithmeticIs instruction correctly evaluates
    /// an arithmetic expression and stores the result in the target register.
    #[test]
    fn test_execute_arithmetic_is_success() {
        // Expression: (3 + 4) * 2 = 14
        let expr = Expression::Mul(
            Box::new(Expression::Add(
                Box::new(Expression::Const(3)),
                Box::new(Expression::Const(4))
            )),
            Box::new(Expression::Const(2))
        );
        let mut machine = Machine::new(1, vec![]);
        // Call execute_arithmetic_is – note that the implementation clones the expression.
        machine.execute_arithmetic_is(0, expr).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(14)));
    }

    /// Test that MultiIndexedCall correctly builds a composite key from multiple registers,
    /// looks up the clause addresses, and sets the program counter appropriately.
    #[test]
    fn test_execute_multi_indexed_call_success() {
        let mut machine = Machine::new(2, vec![]);
        // Set registers for the composite key.
        machine.registers[0] = Some(Term::Const(3));
        machine.registers[1] = Some(Term::Const(4));

        // Create an index table entry for predicate "p" keyed by [Const(3), Const(4)]
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(3), Term::Const(4)], vec![300]);
        machine.index_table.insert("p".to_string(), index_map);

        // Execute the MultiIndexedCall instruction.
        machine.execute_multi_indexed_call("p".to_string(), vec![0, 1]).unwrap();
        // After executing, the program counter should be set to the first clause address (300).
        assert_eq!(machine.pc, 300);
    }

    /// Test that PutStr stores a string term in the specified register.
    #[test]
    fn test_execute_put_str_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_put_str(0, "hello".to_string()).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Str("hello".to_string())));
    }

    /// Test that GetStr successfully unifies when the register already holds the expected string.
    #[test]
    fn test_execute_get_str_success() {
        let mut machine = Machine::new(1, vec![]);
        // Set register 0 to a string term "world"
        machine.registers[0] = Some(Term::Str("world".to_string()));
        // Execute GetStr expecting the same string.
        machine.execute_get_str(0, "world".to_string()).unwrap();
        // The register should still hold "world" after successful unification.
        assert_eq!(machine.registers[0], Some(Term::Str("world".to_string())));
    }

    /// Test that GetStr fails (with a unification error) when the expected string does not match.
    #[test]
    fn test_execute_get_str_unification_failure() {
        let mut machine = Machine::new(1, vec![]);
        // Set register 0 to "foo"
        machine.registers[0] = Some(Term::Str("foo".to_string()));
        let result = machine.execute_get_str(0, "bar".to_string());
        match result {
            Err(MachineError::UnificationFailed(msg)) => {
                assert!(msg.contains("Cannot unify"), "Unexpected error message: {}", msg);
            },
            _ => panic!("Expected UnificationFailed error for GetStr"),
        }
    }

    /// Test that the Move instruction correctly copies the contents of one register to another.
    #[test]
    fn test_execute_move_success() {
        let mut machine = Machine::new(3, vec![]);
        machine.registers[0] = Some(Term::Const(7));
        machine.execute_move(0, 1).expect("execute_move should succeed");
        assert_eq!(machine.registers[1], Some(Term::Const(7)));
    }

    /// Test that the Move instruction returns an error when the source register is out of bounds.
    #[test]
    fn test_execute_move_error() {
        let mut machine = Machine::new(2, vec![]);
        let result = machine.execute_move(5, 1);
        match result {
            Err(MachineError::RegisterOutOfBounds(n)) => assert_eq!(n, 5),
            _ => panic!("Expected RegisterOutOfBounds error for move instruction"),
        }
    }

    /// Test that the Halt instruction stops machine execution.
    #[test]
    fn test_halt_instruction() {
        // Create a machine with a Halt instruction followed by another instruction.
        let code = vec![
            Instruction::Halt,
            Instruction::PutConst { register: 0, value: 100 },
        ];
        let mut machine = Machine::new(1, code);
        // Run the machine; execution should stop when a Halt instruction is encountered.
        machine.run().expect("Machine run should succeed (halt should stop execution)");
        // The run loop breaks when it sees a Halt instruction.
        // Therefore, the PC is expected to remain at the index of the Halt instruction (which is 0).
        assert_eq!(machine.pc, 0);
        // Additionally, the second instruction should not have been executed.
        assert_ne!(machine.registers[0], Some(Term::Const(100)));
    }
}
