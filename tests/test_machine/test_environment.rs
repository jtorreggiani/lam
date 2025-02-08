#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_environment() {
        let code = vec![
            // Allocate an environment with 2 locals.
            Instruction::Allocate { n: 2 },
            // Set local 0 to 42 and local 1 to 99.
            Instruction::SetLocal { index: 0, value: Term::Const(42) },
            Instruction::SetLocal { index: 1, value: Term::Const(99) },
            // Retrieve locals into registers.
            Instruction::GetLocal { index: 0, register: 0 },
            Instruction::GetLocal { index: 1, register: 1 },
            // Deallocate the environment.
            Instruction::Deallocate,
        ];
        
        let mut machine = Machine::new(2, code);
        machine.run().expect("Machine run should succeed");
        
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
        assert_eq!(machine.registers[1], Some(Term::Const(99)));
        assert_eq!(machine.environment_stack.len(), 0);
    }
}
