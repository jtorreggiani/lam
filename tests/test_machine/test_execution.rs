#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use lam::machine::arithmetic::Expression;
    use lam::machine::core::Machine;
    use lam::machine::error_handling::MachineError;
    use lam::machine::frame::Frame;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;
    use lam::machine::choice_point::ChoicePoint;

    // ---------------------------
    // execute_put_const tests
    // ---------------------------
    #[test]
    fn test_execute_put_const_success() {
        let mut machine = Machine::new(2, vec![]);
        machine.execute_put_const(0, 42).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
    }

    #[test]
    fn test_execute_put_const_error_out_of_bounds() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_put_const(5, 42).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(n) => assert_eq!(n, 5),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    // ---------------------------
    // execute_put_var tests
    // ---------------------------
    #[test]
    fn test_execute_put_var_success() {
        let mut machine = Machine::new(2, vec![]);
        machine.execute_put_var(1, 7, "X".to_string()).unwrap();
        assert_eq!(machine.registers[1], Some(Term::Var(7)));
        assert_eq!(machine.variable_names.get(&7), Some(&"X".to_string()));
    }

    #[test]
    fn test_execute_put_var_error_out_of_bounds() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_put_var(5, 1, "Y".to_string()).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(n) => assert_eq!(n, 5),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    // ---------------------------
    // execute_get_const tests
    // ---------------------------
    #[test]
    fn test_execute_get_const_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Const(100));
        machine.execute_get_const(0, 100).unwrap();
        // Unification succeeds and register remains unchanged.
        assert_eq!(machine.registers[0], Some(Term::Const(100)));
    }

    #[test]
    fn test_execute_get_const_failure_unification() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Const(50));
        let err = machine.execute_get_const(0, 100).unwrap_err();
        match err {
            MachineError::UnificationFailed(msg) => assert!(msg.contains("Cannot unify")),
            _ => panic!("Expected UnificationFailed error"),
        }
    }

    #[test]
    fn test_execute_get_const_error_uninitialized() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_get_const(0, 42).unwrap_err();
        match err {
            MachineError::UninitializedRegister(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected UninitializedRegister error"),
        }
    }

    // ---------------------------
    // execute_get_var tests
    // ---------------------------
    #[test]
    fn test_execute_get_var_uninitialized() {
        let mut machine = Machine::new(1, vec![]);
        // When register is uninitialized, GetVar should set it.
        machine.execute_get_var(0, 3, "A".to_string()).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Var(3)));
        assert_eq!(machine.variable_names.get(&3), Some(&"A".to_string()));
    }

    #[test]
    fn test_execute_get_var_with_unification() {
        let mut machine = Machine::new(1, vec![]);
        // Set register 0 to Const(77) so GetVar will unify.
        machine.registers[0] = Some(Term::Const(77));
        machine.execute_get_var(0, 5, "B".to_string()).unwrap();
        // The union–find should bind Var(5) to 77 and update register 0.
        assert_eq!(machine.registers[0], Some(Term::Const(77)));
    }

    #[test]
    fn test_execute_get_var_error_out_of_bounds() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_get_var(5, 1, "X".to_string()).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(reg) => assert_eq!(reg, 5),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    // ---------------------------
    // execute_call tests
    // ---------------------------
    #[test]
    fn test_execute_call_builtin() {
        // Built-in "halt" should set pc = code.len()
        let code = vec![Instruction::Call { predicate: "halt".to_string() }];
        let mut machine = Machine::new(1, code);
        machine.execute_call("halt".to_string()).unwrap();
        assert_eq!(machine.pc, machine.code.len());
    }

    #[test]
    fn test_execute_call_user_defined_success() {
        // Register a dummy user-defined predicate "dummy" with clause at address 0.
        let mut machine = Machine::new(1, vec![Instruction::Proceed]);
        machine.register_predicate("dummy".to_string(), 0);
        machine.execute_call("dummy".to_string()).unwrap();
        // A control frame and a choice point should have been pushed.
        assert_eq!(machine.control_stack.len(), 1);
        assert_eq!(machine.choice_stack.len(), 1);
        // PC should be set to clause address (0).
        assert_eq!(machine.pc, 0);
    }

    #[test]
    fn test_execute_call_error_predicate_not_found() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_call("nonexistent".to_string()).unwrap_err();
        match err {
            MachineError::PredicateNotFound(pred) => assert_eq!(pred, "nonexistent".to_string()),
            _ => panic!("Expected PredicateNotFound error"),
        }
    }

    // ---------------------------
    // execute_proceed tests
    // ---------------------------
    #[test]
    fn test_execute_proceed() {
        let mut machine = Machine::new(1, vec![]);
        machine.control_stack.push(Frame { return_pc: 42 });
        machine.execute_proceed().unwrap();
        assert_eq!(machine.pc, 42);
        assert!(machine.control_stack.is_empty());
    }

    // ---------------------------
    // execute_choice tests
    // ---------------------------
    #[test]
    fn test_execute_choice() {
        let mut machine = Machine::new(1, vec![]);
        let initial_count = machine.choice_stack.len();
        machine.execute_choice(55).unwrap();
        assert_eq!(machine.choice_stack.len(), initial_count + 1);
        let cp = machine.choice_stack.last().unwrap();
        assert_eq!(cp.alternative_clauses, Some(vec![55]));
    }

    // ---------------------------
    // execute_allocate & deallocate tests
    // ---------------------------
    #[test]
    fn test_execute_allocate() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(3).unwrap();
        assert_eq!(machine.environment_stack.len(), 1);
        assert_eq!(machine.environment_stack.last().unwrap().len(), 3);
        assert!(machine.environment_stack.last().unwrap().iter().all(|slot| slot.is_none()));
    }

    #[test]
    fn test_execute_deallocate_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(2).unwrap();
        machine.execute_deallocate().unwrap();
        assert!(machine.environment_stack.is_empty());
    }

    #[test]
    fn test_execute_deallocate_error() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_deallocate().unwrap_err();
        match err {
            MachineError::EnvironmentMissing => {},
            _ => panic!("Expected EnvironmentMissing error"),
        }
    }

    // ---------------------------
    // execute_arithmetic_is tests
    // ---------------------------
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
        machine.execute_arithmetic_is(0, expr).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(14)));
    }

    #[test]
    fn test_execute_arithmetic_is_error_out_of_bounds() {
        let expr = Expression::Const(10);
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_arithmetic_is(5, expr).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(reg) => assert_eq!(reg, 5),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    // ---------------------------
    // execute_set_local tests
    // ---------------------------
    #[test]
    fn test_execute_set_local_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(2).unwrap();
        machine.execute_set_local(1, Term::Const(88)).unwrap();
        let env = machine.environment_stack.last().unwrap();
        assert_eq!(env[1], Some(Term::Const(88)));
    }

    #[test]
    fn test_execute_set_local_error_no_env() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_set_local(0, Term::Const(5)).unwrap_err();
        match err {
            MachineError::EnvironmentMissing => {},
            _ => panic!("Expected EnvironmentMissing error"),
        }
    }

    #[test]
    fn test_execute_set_local_error_index_out_of_bounds() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(1).unwrap();
        let err = machine.execute_set_local(2, Term::Const(5)).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(idx) => assert_eq!(idx, 2),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    // ---------------------------
    // execute_get_local tests
    // ---------------------------
    #[test]
    fn test_execute_get_local_success() {
        let mut machine = Machine::new(2, vec![]);
        machine.execute_allocate(2).unwrap();
        // Set local[0] to Const(77)
        {
            let env = machine.environment_stack.last_mut().unwrap();
            env[0] = Some(Term::Const(77));
        }
        machine.execute_get_local(0, 1).unwrap();
        assert_eq!(machine.registers[1], Some(Term::Const(77)));
    }

    #[test]
    fn test_execute_get_local_error_no_env() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_get_local(0, 0).unwrap_err();
        match err {
            MachineError::EnvironmentMissing => {},
            _ => panic!("Expected EnvironmentMissing error"),
        }
    }

    #[test]
    fn test_execute_get_local_error_uninitialized_local() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(1).unwrap();
        let err = machine.execute_get_local(0, 0).unwrap_err();
        match err {
            MachineError::UninitializedRegister(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected UninitializedRegister error"),
        }
    }

    // ---------------------------
    // execute_fail tests
    // ---------------------------
    #[test]
    fn test_execute_fail_success_backtracking() {
        let mut machine = Machine::new(1, vec![]);
        // Prepare a choice point.
        let cp = ChoicePoint {
            saved_pc: 10,
            saved_registers: vec![Some(Term::Const(5))],
            saved_substitution: HashMap::new(),
            saved_control_stack: vec![Frame { return_pc: 20 }],
            alternative_clauses: Some(vec![30]),
            uf_trail_len: machine.uf.trail.len(),
            call_level: 0,
        };
        machine.choice_stack.push(Box::new(cp));
        // Change the machine state.
        machine.registers[0] = Some(Term::Const(100));
        machine.execute_fail().unwrap();
        // State should be restored from the choice point.
        assert_eq!(machine.registers[0], Some(Term::Const(5)));
        assert_eq!(machine.control_stack.len(), 1);
        assert_eq!(machine.pc, 30);
    }

    #[test]
    fn test_execute_fail_error_no_choice_point() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_fail().unwrap_err();
        match err {
            MachineError::NoChoicePoint => {},
            _ => panic!("Expected NoChoicePoint error"),
        }
    }

    // ---------------------------
    // execute_get_structure tests
    // ---------------------------
    #[test]
    fn test_execute_get_structure_success() {
        let compound = Term::Compound("foo".to_string(), vec![Term::Const(1), Term::Const(2)]);
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(compound);
        machine.execute_get_structure(0, "foo".to_string(), 2).unwrap();
    }

    #[test]
    fn test_execute_get_structure_error_not_compound() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Const(42));
        let err = machine.execute_get_structure(0, "foo".to_string(), 2).unwrap_err();
        match err {
            MachineError::NotACompoundTerm(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected NotACompoundTerm error"),
        }
    }

    #[test]
    fn test_execute_get_structure_error_uninitialized() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_get_structure(0, "foo".to_string(), 2).unwrap_err();
        match err {
            MachineError::UninitializedRegister(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected UninitializedRegister error"),
        }
    }

    // ---------------------------
    // execute_indexed_call tests
    // ---------------------------
    #[test]
    fn test_execute_indexed_call_success() {
        let mut machine = Machine::new(1, vec![]);
        // Set register 0 to Const(7)
        machine.registers[0] = Some(Term::Const(7));
        // Prepare an index table for predicate "p" with key [Const(7)] → [100, 200]
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(7)], vec![100, 200]);
        machine.index_table.insert("p".to_string(), index_map);
        machine.execute_indexed_call("p".to_string(), 0).unwrap();
        // PC should be set to the first clause (100) and a choice point pushed with alternative [200]
        assert_eq!(machine.pc, 100);
        let cp = machine.choice_stack.last().unwrap();
        assert_eq!(cp.alternative_clauses, Some(vec![200]));
    }

    #[test]
    fn test_execute_indexed_call_error_no_index_map() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Const(5));
        let err = machine.execute_indexed_call("nonexistent".to_string(), 0).unwrap_err();
        match err {
            MachineError::PredicateNotInIndex(pred) => assert_eq!(pred, "nonexistent".to_string()),
            _ => panic!("Expected PredicateNotInIndex error"),
        }
    }

    #[test]
    fn test_execute_indexed_call_error_no_index_entry() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Const(5));
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(1)], vec![10]);
        machine.index_table.insert("p".to_string(), index_map);
        let err = machine.execute_indexed_call("p".to_string(), 0).unwrap_err();
        match err {
            MachineError::NoIndexEntry(pred, key) => {
                assert_eq!(pred, "p".to_string());
                assert_eq!(key, Term::Const(5));
            },
            _ => panic!("Expected NoIndexEntry error"),
        }
    }

    // ---------------------------
    // execute_put_str tests
    // ---------------------------
    #[test]
    fn test_execute_put_str_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_put_str(0, "hello".to_string()).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Str("hello".to_string())));
    }

    #[test]
    fn test_execute_put_str_error_out_of_bounds() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_put_str(2, "test".to_string()).unwrap_err();
        match err {
            MachineError::RegisterOutOfBounds(reg) => assert_eq!(reg, 2),
            _ => panic!("Expected RegisterOutOfBounds error"),
        }
    }

    // ---------------------------
    // execute_get_str tests
    // ---------------------------
    #[test]
    fn test_execute_get_str_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Str("world".to_string()));
        machine.execute_get_str(0, "world".to_string()).unwrap();
    }

    #[test]
    fn test_execute_get_str_unification_failure() {
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(Term::Str("foo".to_string()));
        let err = machine.execute_get_str(0, "bar".to_string()).unwrap_err();
        match err {
            MachineError::UnificationFailed(msg) => assert!(msg.contains("Cannot unify")),
            _ => panic!("Expected UnificationFailed error"),
        }
    }

    #[test]
    fn test_execute_get_str_error_uninitialized() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_get_str(0, "test".to_string()).unwrap_err();
        match err {
            MachineError::UninitializedRegister(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected UninitializedRegister error"),
        }
    }

    // ---------------------------
    // execute_multi_indexed_call tests
    // ---------------------------
    #[test]
    fn test_execute_multi_indexed_call_success() {
        let mut machine = Machine::new(2, vec![]);
        machine.registers[0] = Some(Term::Const(3));
        machine.registers[1] = Some(Term::Const(4));
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(3), Term::Const(4)], vec![300]);
        machine.index_table.insert("p".to_string(), index_map);
        machine.execute_multi_indexed_call("p".to_string(), vec![0, 1]).unwrap();
        assert_eq!(machine.pc, 300);
    }

    #[test]
    fn test_execute_multi_indexed_call_error_out_of_bounds() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_multi_indexed_call("p".to_string(), vec![0]).unwrap_err();
        match err {
            MachineError::UninitializedRegister(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected UninitializedRegister error"),
        }
    }

    // ---------------------------
    // execute_tail_call tests
    // ---------------------------
    #[test]
    fn test_execute_tail_call_builtin() {
        // Tail call using built-in "halt" should deallocate environment and set pc accordingly.
        let mut machine = Machine::new(1, vec![
            Instruction::TailCall { predicate: "halt".to_string() },
        ]);
        machine.execute_allocate(1).unwrap();
        machine.execute_tail_call("halt".to_string()).unwrap();
        assert_eq!(machine.pc, machine.code.len());
        assert!(machine.environment_stack.is_empty());
    }

    #[test]
    fn test_execute_tail_call_user_defined() {
        // Tail call for a user-defined predicate "dummy"
        let mut machine = Machine::new(1, vec![
            Instruction::TailCall { predicate: "dummy".to_string() },
        ]);
        machine.execute_allocate(1).unwrap();
        machine.register_predicate("dummy".to_string(), 50);
        // Ensure code is long enough and define a clause at address 50.
        machine.code.resize(51, Instruction::Halt);
        machine.code[50] = Instruction::PutConst { register: 0, value: 555 };
        machine.code.push(Instruction::Proceed);
        // Execute tail call.
        machine.execute_tail_call("dummy".to_string()).unwrap();
        // Now execute the instruction at the new PC (address 50).
        machine.step().unwrap();
        // The environment should have been deallocated and clause executed.
        assert!(machine.environment_stack.is_empty());
        assert_eq!(machine.registers[0], Some(Term::Const(555)));
    }

    #[test]
    fn test_execute_tail_call_error_no_env() {
        let mut machine = Machine::new(1, vec![
            Instruction::TailCall { predicate: "dummy".to_string() },
        ]);
        let err = machine.execute_tail_call("dummy".to_string()).unwrap_err();
        match err {
            MachineError::EnvironmentMissing => {},
            _ => panic!("Expected EnvironmentMissing error"),
        }
    }

    // ---------------------------
    // execute_assert_clause tests
    // ---------------------------
    #[test]
    fn test_execute_assert_clause() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_assert_clause("p".to_string(), 123).unwrap();
        assert!(machine.predicate_table.get("p").unwrap().contains(&123));
        // If an index table exists for "p", it should be updated.
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(10)], vec![10]);
        machine.index_table.insert("p".to_string(), index_map);
        machine.execute_assert_clause("p".to_string(), 456).unwrap();
        let clauses = machine.index_table.get("p").unwrap().get(&vec![Term::Const(10)]).unwrap();
        assert!(clauses.contains(&456));
    }

    // ---------------------------
    // execute_retract_clause tests
    // ---------------------------
    #[test]
    fn test_execute_retract_clause_success() {
        let mut machine = Machine::new(1, vec![]);
        machine.register_predicate("p".to_string(), 777);
        machine.execute_retract_clause("p".to_string(), 777).unwrap();
        assert!(machine.predicate_table.get("p").unwrap().is_empty());
    }

    #[test]
    fn test_execute_retract_clause_error_not_found() {
        let mut machine = Machine::new(1, vec![]);
        machine.register_predicate("p".to_string(), 888);
        let err = machine.execute_retract_clause("p".to_string(), 999).unwrap_err();
        match err {
            MachineError::PredicateClauseNotFound(pred) => assert_eq!(pred, "p".to_string()),
            _ => panic!("Expected PredicateClauseNotFound error"),
        }
    }

    #[test]
    fn test_execute_retract_clause_error_predicate_not_found() {
        let mut machine = Machine::new(1, vec![]);
        let err = machine.execute_retract_clause("nonexistent".to_string(), 123).unwrap_err();
        match err {
            MachineError::PredicateNotFound(pred) => assert_eq!(pred, "nonexistent".to_string()),
            _ => panic!("Expected PredicateNotFound error"),
        }
    }

    // ---------------------------
    // execute_cut tests
    // ---------------------------
    #[test]
    fn test_execute_cut() {
        let mut machine = Machine::new(1, vec![]);
        // Simulate a control stack of length 2.
        machine.control_stack.push(Frame { return_pc: 10 });
        machine.control_stack.push(Frame { return_pc: 20 });
        // Add two choice points with different call levels.
        let cp1 = ChoicePoint {
            saved_pc: 5,
            saved_registers: vec![],
            saved_substitution: HashMap::new(),
            saved_control_stack: vec![],
            alternative_clauses: Some(vec![5]),
            uf_trail_len: 0,
            call_level: 1,
        };
        let cp2 = ChoicePoint {
            saved_pc: 6,
            saved_registers: vec![],
            saved_substitution: HashMap::new(),
            saved_control_stack: vec![],
            alternative_clauses: Some(vec![6]),
            uf_trail_len: 0,
            call_level: 2,
        };
        machine.choice_stack.push(Box::new(cp1));
        machine.choice_stack.push(Box::new(cp2));
        machine.execute_cut().unwrap();
        // Only choice points with call_level < current (2) should remain.
        assert_eq!(machine.choice_stack.len(), 1);
        let remaining_cp = &machine.choice_stack[0];
        assert_eq!(remaining_cp.call_level, 1);
    }

    // ---------------------------
    // execute_build_compound tests
    // ---------------------------
    #[test]
    fn test_execute_build_compound_success() {
        let mut machine = Machine::new(3, vec![]);
        machine.registers[0] = Some(Term::Const(10));
        machine.registers[1] = Some(Term::Const(20));
        machine.execute_build_compound(2, "f".to_string(), vec![0, 1]).unwrap();
        let expected = Term::Compound("f".to_string(), vec![Term::Const(10), Term::Const(20)]);
        assert_eq!(machine.registers[2], Some(expected));
    }

    #[test]
    fn test_execute_build_compound_error_arg_register_uninitialized() {
        let mut machine = Machine::new(3, vec![]);
        // Register 0 is uninitialized.
        machine.registers[1] = Some(Term::Const(20));
        let err = machine.execute_build_compound(2, "f".to_string(), vec![0, 1]).unwrap_err();
        match err {
            MachineError::UninitializedRegister(reg) => assert_eq!(reg, 0),
            _ => panic!("Expected UninitializedRegister error"),
        }
    }
}
