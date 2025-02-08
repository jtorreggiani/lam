#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_tail_call() {
        // Main program:
        let code = vec![
            // Allocate an environment frame.
            Instruction::Allocate { n: 1 },
            // Set local variable 0 to 100.
            Instruction::SetLocal { index: 0, value: Term::Const(100) },
            // Tail call to predicate "p" (this should deallocate the current environment).
            Instruction::TailCall { predicate: "p".to_string() },
            // Dummy instruction (should not execute if tail call works).
            Instruction::PutConst { register: 0, value: 999 },
            // Predicate "p" code (starting at index 4).
            Instruction::PutConst { register: 0, value: 200 },
            // Return.
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(1, code);
        
        // Register predicate "p" to start at index 4.
        machine.register_predicate("p".to_string(), 4);
        machine.run().expect("Machine run should succeed");
        
        // Expect that reg0 was set to 200 and the environment stack is empty.
        assert_eq!(machine.registers[0], Some(Term::Const(200)));
        assert_eq!(machine.environment_stack.len(), 0);
    }
}
