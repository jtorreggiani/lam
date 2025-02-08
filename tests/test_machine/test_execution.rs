#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lam::machine::arithmetic::Expression;
    use lam::machine::core::Machine;
    use lam::machine::error_handling::MachineError;
    use lam::machine::frame::Frame;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_execute_put_const() {
        let code = vec![Instruction::PutConst { register: 0, value: 42 }];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
    }

    #[test]
    fn test_execute_put_var() {
        let code = vec![Instruction::PutVar { register: 0, var_id: 5, name: "V".to_string() }];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Var(5)));
        assert_eq!(machine.variable_names.get(&5), Some(&"V".to_string()));
    }

    #[test]
    fn test_execute_get_const_success() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 99 },
            Instruction::GetConst { register: 0, value: 99 },
        ];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(99)));
    }

    #[test]
    fn test_execute_get_const_failure() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 99 },
            Instruction::GetConst { register: 0, value: 100 },
        ];
        let mut machine = Machine::new(1, code);
        let res = machine.run();
        match res {
            Err(MachineError::UnificationFailed(_)) => {},
            _ => panic!("Expected unification failure"),
        }
    }

    #[test]
    fn test_execute_get_var_uninitialized() {
        // When the register is uninitialized, GetVar should set it to a variable.
        let code = vec![
            Instruction::GetVar { register: 0, var_id: 3, name: "X".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Var(3)));
        assert_eq!(machine.variable_names.get(&3), Some(&"X".to_string()));
    }

    #[test]
    fn test_execute_get_var_unification() {
        // When register holds a term, GetVar should unify it with a variable.
        let code = vec![
            Instruction::PutConst { register: 0, value: 123 },
            Instruction::GetVar { register: 0, var_id: 7, name: "Y".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        // The unification should bind Var(7) to 123.
        assert_eq!(machine.uf.resolve(&Term::Var(7)), Term::Const(123));
    }

    #[test]
    fn test_execute_call_builtin() {
        // Test a built–in predicate: using "halt" to stop execution.
        let code = vec![
            Instruction::Call { predicate: "halt".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        // After halting, pc should be set to code.len()
        assert_eq!(machine.pc, machine.code.len());
    }

    #[test]
    fn test_execute_call_user_defined() {
        // Register a dummy predicate "dummy" that sets register 0 to 777.
        let code = vec![
            Instruction::Call { predicate: "dummy".to_string() },
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(1, code);
        machine.register_predicate("dummy".to_string(), 10);
        // Extend the code so that at address 10 we set reg0 to 777.
        machine.code.resize(11, Instruction::Halt);
        machine.code[10] = Instruction::PutConst { register: 0, value: 777 };
        machine.code.push(Instruction::Proceed);
        machine.run().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(777)));
    }

    #[test]
    fn test_execute_proceed() {
        let code = vec![Instruction::Proceed];
        let mut machine = Machine::new(1, code);
        machine.control_stack.push(Frame { return_pc: 42 });
        machine.execute_proceed().expect("execute_proceed should succeed");
        assert_eq!(machine.pc, 42);
        assert!(machine.control_stack.is_empty());
    }

    #[test]
    fn test_execute_choice() {
        let mut machine = Machine::new(1, vec![]);
        let initial_choice_stack_len = machine.choice_stack.len();
        machine.execute_choice(99).expect("execute_choice should succeed");
        assert_eq!(machine.choice_stack.len(), initial_choice_stack_len + 1);
        let cp = machine.choice_stack.last().unwrap();
        assert_eq!(cp.alternative_clauses, Some(vec![99]));
    }

    #[test]
    fn test_execute_allocate_deallocate() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(3).expect("allocate should succeed");
        assert_eq!(machine.environment_stack.len(), 1);
        assert_eq!(machine.environment_stack.last().unwrap().len(), 3);
        machine.execute_deallocate().expect("deallocate should succeed");
        assert!(machine.environment_stack.is_empty());
    }

    #[test]
    fn test_execute_arithmetic_is() {
        // Evaluate (3 + 4) * 2 = 14.
        let expr = Expression::Mul(
            Box::new(Expression::Add(Box::new(Expression::Const(3)), Box::new(Expression::Const(4)))),
            Box::new(Expression::Const(2))
        );
        let code = vec![Instruction::ArithmeticIs { target: 0, expression: expr }];
        let mut machine = Machine::new(1, code);
        machine.run().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(14)));
    }

    #[test]
    fn test_execute_set_local_and_get_local() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_allocate(2).expect("allocate should succeed");
        machine.execute_set_local(0, Term::Const(55)).expect("set_local should succeed");
        machine.execute_get_local(0, 0).expect("get_local should succeed");
        assert_eq!(machine.registers[0], Some(Term::Const(55)));
        machine.execute_deallocate().expect("deallocate should succeed");
    }

    #[test]
    fn test_execute_tail_call() {
        // Build a small program with:
        // Index 0: TailCall to predicate "p"
        // Index 1: Halt (this instruction won't be executed because tail call jumps to the target)
        // Index 2: PutConst reg0, 555  (the body of predicate "p")
        // Index 3: Proceed
        let code = vec![
            Instruction::TailCall { predicate: "p".to_string() },
            Instruction::Halt,
            Instruction::PutConst { register: 0, value: 555 },
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(1, code);
        // Ensure there is an environment frame so that tail call can pop it.
        machine.execute_allocate(1).unwrap();
        // Register the predicate "p" with a clause address of 2.
        machine.register_predicate("p".to_string(), 2);
        // Run the machine. The TailCall instruction will jump to clause address 2.
        machine.run().unwrap();
        // After tail call execution, the environment should have been deallocated.
        assert_eq!(machine.environment_stack.len(), 0);
        // The target clause (index 2) should have been executed,
        // and after executing the clause (plus the subsequent Proceed),
        // the machine should have halted with pc equal to the code length.
        assert_eq!(machine.pc, machine.code.len());
        // Verify that register 0 is set to 555 (the value produced by the target clause).
        assert_eq!(machine.registers[0], Some(Term::Const(555)));
    }

    #[test]
    fn test_execute_get_structure() {
        let compound = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);
        let code = vec![Instruction::PutConst { register: 0, value: 0 }]; // dummy instruction
        let mut machine = Machine::new(1, code);
        machine.registers[0] = Some(compound.clone());
        machine.execute_get_structure(0, "f".to_string(), 2).expect("get_structure should succeed");
        machine.registers[0] = Some(Term::Compound("g".to_string(), vec![Term::Const(1), Term::Const(2)]));
        let res = machine.execute_get_structure(0, "f".to_string(), 2);
        match res {
            Err(MachineError::StructureMismatch { expected_functor, expected_arity, found_functor, found_arity }) => {
                assert_eq!(expected_functor, "f");
                assert_eq!(expected_arity, 2);
                assert_eq!(found_functor, "g");
                assert_eq!(found_arity, 2);
            },
            _ => panic!("Expected StructureMismatch error"),
        }
    }

    #[test]
    fn test_execute_indexed_call() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(1, code);
        let mut dummy_index: HashMap<Vec<Term>, Vec<usize>> = HashMap::new();
        dummy_index.insert(vec![Term::Const(1)], vec![42]);
        machine.index_table.insert("p".to_string(), dummy_index);
        machine.run().expect("Machine run should succeed");
        assert_eq!(machine.pc, 42);
    }

    #[test]
    fn test_execute_put_str_get_str() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_put_str(0, "hello".to_string()).expect("put_str should succeed");
        assert_eq!(machine.registers[0], Some(Term::Str("hello".to_string())));
        machine.execute_get_str(0, "hello".to_string()).expect("get_str should succeed");
        machine.registers[0] = Some(Term::Str("world".to_string()));
        let res = machine.execute_get_str(0, "hello".to_string());
        match res {
            Err(MachineError::UnificationFailed(_)) => {},
            _ => panic!("Expected unification failure"),
        }
    }

    #[test]
    fn test_execute_multi_indexed_call() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 2 },
            Instruction::PutConst { register: 1, value: 3 },
            Instruction::MultiIndexedCall { predicate: "p".to_string(), index_registers: vec![0, 1] },
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(2, code);
        let mut index_map: HashMap<Vec<Term>, Vec<usize>> = HashMap::new();
        index_map.insert(vec![Term::Const(2), Term::Const(3)], vec![55]);
        machine.index_table.insert("p".to_string(), index_map);
        machine.run().expect("Machine run should succeed");
        assert_eq!(machine.pc, 55);
    }

    #[test]
    fn test_execute_assert_and_retract_clause() {
        let mut machine = Machine::new(1, vec![]);
        machine.execute_assert_clause("p".to_string(), 100).expect("assert_clause should succeed");
        {
            let clauses = machine.predicate_table.get("p").unwrap();
            assert!(clauses.contains(&100));
        }
        machine.execute_retract_clause("p".to_string(), 100).expect("retract_clause should succeed");
        let empty = machine.predicate_table.get("p").map(|v| v.is_empty()).unwrap_or(true);
        assert!(empty);
    }

    #[test]
    fn test_execute_cut_instruction() {
        // Setup a machine with two choice points.
        let code = vec![
            Instruction::Choice { alternative: 10 },
            Instruction::Choice { alternative: 20 },
            Instruction::Cut,
        ];
        let mut machine = Machine::new(1, code);
        // Simulate a deeper call level.
        machine.control_stack.push(Frame { return_pc: 0 });
        machine.execute_cut().expect("execute_cut should succeed");
        for cp in &machine.choice_stack {
            assert!(cp.call_level < machine.control_stack.len());
        }
    }

    #[test]
    fn test_execute_build_compound() {
        let mut machine = Machine::new(3, vec![]);
        machine.registers[0] = Some(Term::Const(5));
        machine.registers[1] = Some(Term::Const(10));
        machine.execute_build_compound(2, "f".to_string(), vec![0, 1]).expect("build_compound should succeed");
        let expected = Term::Compound("f".to_string(), vec![Term::Const(5), Term::Const(10)]);
        assert_eq!(machine.registers[2], Some(expected));
    }
}
