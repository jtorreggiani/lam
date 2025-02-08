#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_indexed_call() {
        let code = vec![
            // Main program: perform an IndexedCall.
            Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
            // Clause for predicate "p" when key is [Const(1)] (address 1).
            Instruction::PutConst { register: 0, value: 10 },
            Instruction::Fail,
            // Clause for predicate "p" when key is [Const(2)] (address 3).
            Instruction::PutConst { register: 0, value: 20 },
            Instruction::Proceed,
        ];
        
        let mut machine = Machine::new(1, code);
        
        // Pre-populate the index table.
        machine.register_indexed_clause("p".to_string(), vec![Term::Const(1)], 1);
        machine.register_indexed_clause("p".to_string(), vec![Term::Const(2)], 3);
        
        // Set register 0 to key [Const(2)].
        machine.registers[0] = Some(Term::Const(2));
        
        machine.run().expect("Machine run should succeed");
        
        // Expect that Clause for key [Const(2)] was chosen, so reg0 becomes 20.
        assert_eq!(machine.registers[0], Some(Term::Const(20)));
    }
}
