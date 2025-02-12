// tests/test_eq_builtin.rs

#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;

    /// Test that the equality built–in succeeds when unifying two identical constants.
    #[test]
    fn test_builtin_eq_success() {
        // Create a simple program that calls the equality built–in.
        // We assume that the equality built–in expects its left argument in register 0
        // and its right argument in register 1.
        let code = vec![
            Instruction::Call { predicate: "=".to_string() },
        ];
        let mut machine = Machine::new(2, code);
        // Place two identical constants in registers 0 and 1.
        machine.registers[0] = Some(Term::Const(42));
        machine.registers[1] = Some(Term::Const(42));
        // Execute the equality built–in.
        machine.step().expect("Unification should succeed");
        // Verify that both registers remain unchanged.
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
        assert_eq!(machine.registers[1], Some(Term::Const(42)));
    }

    /// Test that the equality built–in fails when unifying two different constants.
    #[test]
    fn test_builtin_eq_failure() {
        let code = vec![
            Instruction::Call { predicate: "=".to_string() },
        ];
        let mut machine = Machine::new(2, code);
        machine.registers[0] = Some(Term::Const(42));
        machine.registers[1] = Some(Term::Const(43));
        let result = machine.step();
        match result {
            Err(MachineError::UnificationFailed(_)) => { /* Expected failure */ },
            _ => panic!("Expected unification failure due to mismatched constants"),
        }
    }

    /// Test a small program that simulates variable assignment.
    ///
    /// The program corresponds roughly to:
    /// 
    /// ```prolog
    /// main :-
    ///     X = 'Hello world',
    ///     write(X),
    ///     halt.
    /// ```
    ///
    /// The compiler (or a manual simulation thereof) is assumed to generate code that:
    ///   - Puts variable X in register 0.
    ///   - Puts the string 'Hello world' in register 1.
    ///   - Calls the equality built–in "=" to unify the two.
    ///   - Calls "write" (which prints the contents of register 0).
    ///   - Halts the machine.
    #[test]
    fn test_variable_assignment_program() {
        let code = vec![
            // Put variable X into register 0.
            Instruction::PutVar { register: 0, var_id: 0, name: "X".to_string() },
            // Put the string 'Hello world' into register 1.
            Instruction::PutStr { register: 1, value: "Hello world".to_string() },
            // Call the equality built–in to unify register 0 and register 1.
            Instruction::Call { predicate: "=".to_string() },
            // Call the built–in "write" to print the value of X (now bound).
            Instruction::Call { predicate: "write".to_string() },
            // Halt the machine.
            Instruction::Halt,
        ];
        let mut machine = Machine::new(2, code);
        machine.run().expect("Machine run should succeed");
        // Check that unification was successful:
        // The union–find structure should now resolve Var(0) to the string 'Hello world'.
        let resolved = machine.uf.resolve(&Term::Var(0));
        assert_eq!(resolved, Term::Str("Hello world".to_string()));
    }
}
