#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_put_const_instruction() {
        let code = vec![Instruction::PutConst { register: 0, value: 42 }];
        let mut machine = Machine::new(2, code);

        // Initially, registers should be None.
        assert_eq!(machine.registers[0], None);
        assert_eq!(machine.registers[1], None);

        machine.step().expect("Step should succeed");
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
        assert_eq!(machine.registers[1], None);

        // No more instructions; step should return an error.
        assert!(machine.step().is_err());
    }

    #[test]
    fn test_put_var_instruction() {
        let code = vec![Instruction::PutVar { register: 1, var_id: 0, name: "X".to_string() }];
        let mut machine = Machine::new(2, code);
        machine.run().expect("Machine run should succeed");
        assert_eq!(machine.registers[1], Some(Term::Var(0)));
    }

    #[test]
    fn test_get_const_unification_success() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 42 },
            Instruction::GetConst { register: 0, value: 42 },
        ];
        let mut machine = Machine::new(1, code);
        machine.run().expect("Machine run should succeed");
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
    }

    #[test]
    fn test_get_const_unification_failure() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 42 },
            Instruction::GetConst { register: 0, value: 100 },
        ];
        let mut machine = Machine::new(1, code);
        assert!(machine.run().is_err());
    }

    #[test]
    fn test_get_var_unification() {
        let code = vec![
            Instruction::PutVar { register: 0, var_id: 0, name: "X".to_string() },
            Instruction::GetVar { register: 0, var_id: 1, name: "Y".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.run().expect("Machine run should succeed");

        // After unification, register 0 should remain Var(0) and Var(1) should resolve to Var(0).
        assert_eq!(machine.registers[0], Some(Term::Var(0)));
        assert_eq!(machine.uf.resolve(&Term::Var(1)), Term::Var(0));
    }

    #[test]
    fn test_call_and_proceed() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 10 },
            Instruction::Call { predicate: "dummy".to_string() },
            Instruction::PutConst { register: 1, value: 20 },
            Instruction::Proceed,
            Instruction::PutConst { register: 2, value: 30 },
        ];
        
        let mut machine = Machine::new(3, code);
        machine.register_predicate("dummy".to_string(), 2);
        machine.run().expect("Machine run should succeed");
        
        assert_eq!(machine.registers[0], Some(Term::Const(10)));
        assert_eq!(machine.registers[1], Some(Term::Const(20)));
        assert_eq!(machine.registers[2], Some(Term::Const(30)));
        assert_eq!(machine.control_stack.len(), 0);
    }
}
