#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;

    #[test]
    fn test_new_initialization() {
        let code = vec![Instruction::Halt];
        let machine = Machine::new(5, code.clone());
        // Verify proper initialization
        assert_eq!(machine.registers.len(), 5);
        assert_eq!(machine.code, code);
        assert_eq!(machine.pc, 0);
        assert!(machine.substitution.is_empty());
        assert!(machine.control_stack.is_empty());
        assert!(machine.choice_stack.is_empty());
        assert!(machine.environment_stack.is_empty());
        assert!(machine.index_table.is_empty());
        assert!(machine.variable_names.is_empty());
        assert!(machine.uf.bindings.is_empty());
        assert!(!machine.verbose);
        // Verify builtins exist
        assert!(machine.builtins.contains_key("halt"));
        assert!(machine.builtins.contains_key("print"));
        assert!(machine.builtins.contains_key("print_subst"));
        assert!(machine.builtins.contains_key("write"));
        assert!(machine.builtins.contains_key("nl"));
    }

    #[test]
    fn test_register_predicate() {
        let mut machine = Machine::new(1, vec![]);
        machine.register_predicate("p".to_string(), 10);
        machine.register_predicate("p".to_string(), 20);
        assert!(machine.predicate_table.contains_key("p"));
        let clauses = machine.predicate_table.get("p").unwrap();
        assert_eq!(clauses.len(), 2);
        assert!(clauses.contains(&10));
        assert!(clauses.contains(&20));
    }

    #[test]
    fn test_register_indexed_clause() {
        let mut machine = Machine::new(1, vec![]);
        let key = vec![Term::Const(5)];
        machine.register_indexed_clause("p".to_string(), key.clone(), 42);
        assert!(machine.index_table.contains_key("p"));
        let index_map = machine.index_table.get("p").unwrap();
        assert!(index_map.contains_key(&key));
        let clauses = index_map.get(&key).unwrap();
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0], 42);
    }

    #[test]
    fn test_execute_move() {
        let mut machine = Machine::new(3, vec![]);
        machine.registers[0] = Some(Term::Const(7));
        machine.execute_move(0, 1).expect("execute_move should succeed");
        assert_eq!(machine.registers[1], Some(Term::Const(7)));
    }

    #[test]
    fn test_execute_move_error() {
        let mut machine = Machine::new(2, vec![]);
        let err = machine.execute_move(5, 1).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(n) => assert_eq!(n, 5),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    #[test]
    fn test_unify_constants() {
        let mut machine = Machine::new(1, vec![]);
        // Unify two identical constants.
        machine.unify(&Term::Const(10), &Term::Const(10))
            .expect("Unification should succeed");
        // Unify two different constants should fail.
        let err = machine.unify(&Term::Const(10), &Term::Const(20)).unwrap_err();
        match err {
            MachineError::UnificationFailed(ref msg) => {
                assert!(msg.contains("Constants do not match"), "Unexpected message: {}", msg);
            },
            _ => panic!("Expected UnificationFailed error"),
        }
    }

    #[test]
    fn test_builtin_halt() {
        let code = vec![
            Instruction::Halt,
            Instruction::PutConst { register: 0, value: 100 },
        ];
        let mut machine = Machine::new(1, code);
        machine.builtin_halt().expect("builtin_halt should succeed");
        // After calling builtin_halt, the implementation sets pc = code.len() (here, 2).
        assert_eq!(machine.pc, machine.code.len());
    }

    #[test]
    fn test_builtin_print_subst() {
        let mut machine = Machine::new(1, vec![]);
        // Set a substitution and variable name.
        machine.substitution.insert(0, Term::Const(42));
        machine.variable_names.insert(0, "X".to_string());
        // Calling builtin_print_subst should succeed (output not captured).
        machine.builtin_print_subst().expect("builtin_print_subst should succeed");
    }

    #[test]
    fn test_builtin_write() {
        let mut machine = Machine::new(1, vec![]);
        // Set register 0 to a compound with functor "-" and two arguments.
        let compound = Term::Compound("-".to_string(), vec![Term::Const(3), Term::Const(4)]);
        machine.registers[0] = Some(compound);
        machine.builtin_write().expect("builtin_write should succeed");
    }

    #[test]
    fn test_builtin_nl() {
        let mut machine = Machine::new(1, vec![]);
        machine.builtin_nl().expect("builtin_nl should succeed");
    }

    #[test]
    fn test_trace() {
        let mut machine = Machine::new(1, vec![Instruction::Halt]);
        machine.verbose = true;
        // Calling trace on a Halt instruction. (Output goes to log.)
        machine.trace(&Instruction::Halt);
    }

    #[test]
    fn test_step_and_run() {
        // Create a machine with two instructions: PutConst then Halt.
        let code = vec![
            Instruction::PutConst { register: 0, value: 77 },
            Instruction::Halt,
        ];
        let mut machine = Machine::new(1, code.clone());
        // Step once.
        machine.step().expect("step should succeed");
        assert_eq!(machine.registers[0], Some(Term::Const(77)));
        // Now run() will see the Halt instruction and break out.
        machine.run().expect("run should succeed");
        // Depending on your implementation of run(), it might leave pc at the Halt instruction index.
        // (If run() breaks on Halt without updating pc, then pc remains at index 1.)
        // Here, we assert that pc is 1.
        assert_eq!(machine.pc, 1);
    }
}
