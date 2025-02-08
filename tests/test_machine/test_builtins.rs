// tests/test_machine/test_builtins.rs

#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::term::Term;
    use lam::machine::instruction::Instruction;

    #[test]
    fn test_builtin_print() {
        // Create a machine with two registers.
        let mut machine = Machine::new(2, vec![]);
        // Set register 0 to a variable (with name "X") and register 1 to a constant.
        machine.registers[0] = Some(Term::Var(0));
        machine.variable_names.insert(0, "X".to_string());
        machine.registers[1] = Some(Term::Const(42));

        // Enable verbose mode so that builtin_print actually logs output.
        machine.verbose = true;
        // Call builtin_print; we simply assert it returns Ok(()).
        assert!(machine.builtin_print().is_ok());
    }

    #[test]
    fn test_builtin_write_dash_compound() {
        // Test builtin_write when register 0 holds a compound with functor "-" and exactly two arguments.
        let mut machine = Machine::new(1, vec![]);
        let compound = Term::Compound("-".to_string(), vec![Term::Const(3), Term::Const(4)]);
        machine.registers[0] = Some(compound);
        // Expect builtin_write to succeed.
        assert!(machine.builtin_write().is_ok());
    }

    #[test]
    fn test_builtin_write_non_dash() {
        // Test builtin_write when register 0 holds a non-special term.
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Const(99));
        // Expect builtin_write to succeed (printing the term normally).
        assert!(machine.builtin_write().is_ok());
    }

    #[test]
    fn test_builtin_nl() {
        // Test builtin_nl simply returns Ok(()). (Output is printed to stdout.)
        let mut machine = Machine::new(1, vec![]);
        assert!(machine.builtin_nl().is_ok());
    }
}
