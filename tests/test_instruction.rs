#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use lam::machine::arithmetic::Expression;
    use lam::machine::choice_point::ChoicePoint;
    use lam::machine::core::Machine;
    use lam::machine::error_handling::MachineError;
    use lam::machine::frame::Frame;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    // === Basic Instructions ===

    #[test]
    fn test_put_const_instruction() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 42 },
        ];
        let mut machine = Machine::new(2, code);
        machine.step().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(42)));
    }

    #[test]
    fn test_put_var_instruction() {
        let code = vec![
            Instruction::PutVar { register: 1, var_id: 5, name: "X".to_string() },
        ];
        let mut machine = Machine::new(3, code);
        machine.step().unwrap();
        assert_eq!(machine.registers[1], Some(Term::Var(5)));
        assert_eq!(machine.variable_names.get(&5), Some(&"X".to_string()));
    }

    #[test]
    fn test_get_const_instruction_success() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 100 },
            Instruction::GetConst { register: 0, value: 100 },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // PutConst
        machine.step().unwrap(); // GetConst
        // The term unifies so register remains unchanged.
        assert_eq!(machine.registers[0], Some(Term::Const(100)));
    }

    #[test]
    fn test_get_const_instruction_failure() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 100 },
            Instruction::GetConst { register: 0, value: 200 },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // PutConst
        let result = machine.step();
        match result {
            Err(MachineError::UnificationFailed(_)) => {},
            _ => panic!("Expected UnificationFailed error"),
        }
    }

    #[test]
    fn test_get_var_instruction_uninitialized() {
        let code = vec![
            Instruction::GetVar { register: 0, var_id: 10, name: "Y".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap();
        // Since the register was uninitialized, it should now be set to Var(10)
        assert_eq!(machine.registers[0], Some(Term::Var(10)));
        assert_eq!(machine.variable_names.get(&10), Some(&"Y".to_string()));
    }

    #[test]
    fn test_get_var_instruction_with_unification() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 77 },
            Instruction::GetVar { register: 0, var_id: 20, name: "Z".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // PutConst
        machine.step().unwrap(); // GetVar (should unify Var(20) with Const(77))
        // The register remains Const(77) and union-find binds Var(20) accordingly.
        assert_eq!(machine.registers[0], Some(Term::Const(77)));
    }

    // === Arithmetic Instruction ===

    #[test]
    fn test_instruction_arithmetic_is_variant() {
        // Create an ArithmeticIs instruction using a subtraction expression: 10 - 3 = 7.
        let expr = Expression::Sub(
            Box::new(Expression::Const(10)),
            Box::new(Expression::Const(3))
        );
        let instr = Instruction::ArithmeticIs { target: 0, expression: expr.clone() };
        let mut machine = Machine::new(1, vec![instr.clone()]);
        // Execute the instruction via the execute() method.
        instr.execute(&mut machine).unwrap();
        assert_eq!(machine.registers[0], Some(Term::Const(7)));
    }

    // === Control Flow Instructions ===

    #[test]
    fn test_call_instruction_builtin() {
        // Calling built-in "halt" should stop execution.
        let code = vec![
            Instruction::Call { predicate: "halt".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap();
        // For a built-in, the implementation sets PC to code.len() (i.e. 1).
        assert_eq!(machine.pc, machine.code.len());
    }

    #[test]
    fn test_call_instruction_user_defined() {
        // Test a user-defined call: register a predicate "dummy" with two clause addresses.
        let code = vec![
            Instruction::Call { predicate: "dummy".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.register_predicate("dummy".to_string(), 5);
        machine.register_predicate("dummy".to_string(), 10);
        machine.step().unwrap();
        // A control frame should be pushed and a choice point added.
        assert_eq!(machine.control_stack.len(), 1);
        assert_eq!(machine.choice_stack.len(), 1);
        // The PC is set to the first clause address.
        assert_eq!(machine.pc, 5);
    }

    #[test]
    fn test_proceed_instruction() {
        let code = vec![
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(1, code);
        // Simulate a call by pushing a frame.
        machine.control_stack.push(Frame { return_pc: 42 });
        machine.step().unwrap();
        assert_eq!(machine.pc, 42);
        assert!(machine.control_stack.is_empty());
    }

    #[test]
    fn test_choice_instruction() {
        let code = vec![
            Instruction::Choice { alternative: 55 },
        ];
        let mut machine = Machine::new(1, code);
        let initial_choices = machine.choice_stack.len();
        machine.step().unwrap();
        assert_eq!(machine.choice_stack.len(), initial_choices + 1);
        let cp = machine.choice_stack.last().unwrap();
        assert_eq!(cp.alternative_clauses, Some(vec![55]));
    }

    // === Environment and Local Variable Instructions ===

    #[test]
    fn test_allocate_instruction() {
        let code = vec![
            Instruction::Allocate { n: 3 },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap();
        assert_eq!(machine.environment_stack.len(), 1);
        let env = machine.environment_stack.last().unwrap();
        assert_eq!(env.len(), 3);
        for slot in env {
            assert!(slot.is_none());
        }
    }

    #[test]
    fn test_deallocate_instruction_success() {
        let code = vec![
            Instruction::Deallocate,
        ];
        let mut machine = Machine::new(1, code);
        machine.environment_stack.push(vec![Some(Term::Const(1))]);
        machine.step().unwrap();
        assert!(machine.environment_stack.is_empty());
    }

    #[test]
    fn test_deallocate_instruction_error() {
        let code = vec![
            Instruction::Deallocate,
        ];
        let mut machine = Machine::new(1, code);
        let result = machine.step();
        match result {
            Err(MachineError::EnvironmentMissing) => {},
            _ => panic!("Expected EnvironmentMissing error"),
        }
    }

    #[test]
    fn test_set_local_instruction() {
        let code = vec![
            Instruction::Allocate { n: 2 },
            Instruction::SetLocal { index: 1, value: Term::Const(99) },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // Allocate
        machine.step().unwrap(); // SetLocal
        let env = machine.environment_stack.last().unwrap();
        assert_eq!(env[1], Some(Term::Const(99)));
    }

    #[test]
    fn test_get_local_instruction() {
        let code = vec![
            Instruction::Allocate { n: 2 },
            Instruction::SetLocal { index: 0, value: Term::Const(77) },
            Instruction::GetLocal { index: 0, register: 0 },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // Allocate
        machine.step().unwrap(); // SetLocal
        machine.step().unwrap(); // GetLocal
        assert_eq!(machine.registers[0], Some(Term::Const(77)));
    }

    // === Failure and Backtracking Instructions ===

    #[test]
    fn test_fail_instruction() {
        // Prepare a machine with a choice point.
        let code = vec![
            Instruction::Fail,
        ];
        let mut machine = Machine::new(1, code);
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
        // Modify register state.
        machine.registers[0] = Some(Term::Const(100));
        machine.step().unwrap();
        // After failure, state should be restored.
        assert_eq!(machine.registers[0], Some(Term::Const(5)));
        assert_eq!(machine.control_stack.len(), 1);
        assert_eq!(machine.pc, 30);
    }

    #[test]
    fn test_get_structure_instruction_success() {
        // Build a compound term f(1, 2) and check structure.
        let compound = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);
        let code = vec![
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::PutConst { register: 1, value: 2 },
            Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
            Instruction::GetStructure { register: 2, functor: "f".to_string(), arity: 2 },
        ];
        let mut machine = Machine::new(3, code);
        machine.step().unwrap(); // PutConst reg0
        machine.step().unwrap(); // PutConst reg1
        machine.step().unwrap(); // BuildCompound into reg2
        machine.step().unwrap(); // GetStructure
        assert_eq!(machine.registers[2], Some(compound));
    }

    // === Clause Indexing Instructions ===

    #[test]
    fn test_indexed_call_instruction_success() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 7 },
            Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
        ];
        let mut machine = Machine::new(1, code);
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(7)], vec![100]);
        machine.index_table.insert("p".to_string(), index_map);
        machine.step().unwrap(); // PutConst sets register 0
        machine.step().unwrap(); // IndexedCall should update PC
        assert_eq!(machine.pc, 100);
    }

    #[test]
    fn test_multi_indexed_call_instruction_success() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 3 },
            Instruction::PutConst { register: 1, value: 4 },
            Instruction::MultiIndexedCall { predicate: "p".to_string(), index_registers: vec![0, 1] },
        ];
        let mut machine = Machine::new(2, code);
        machine.step().unwrap(); // PutConst reg0
        machine.step().unwrap(); // PutConst reg1
        let mut index_map = HashMap::new();
        index_map.insert(vec![Term::Const(3), Term::Const(4)], vec![300]);
        machine.index_table.insert("p".to_string(), index_map);
        machine.step().unwrap(); // MultiIndexedCall
        assert_eq!(machine.pc, 300);
    }

    // === Tail Call Instructions ===

    #[test]
    fn test_tail_call_instruction_builtin() {
        // TailCall with built-in "halt" deallocates environment and halts.
        let code = vec![
            Instruction::Allocate { n: 1 },
            Instruction::TailCall { predicate: "halt".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // Allocate
        assert_eq!(machine.environment_stack.len(), 1);
        machine.step().unwrap(); // TailCall: built-in "halt"
        assert!(machine.environment_stack.is_empty());
        assert_eq!(machine.pc, machine.code.len());
    }

    #[test]
    fn test_tail_call_instruction_user_defined() {
        let code = vec![
            Instruction::Allocate { n: 1 },
            Instruction::TailCall { predicate: "dummy".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // Allocate
        machine.register_predicate("dummy".to_string(), 50);
        // Extend code so that address 50 is valid.
        machine.code.resize(51, Instruction::Halt);
        machine.code[50] = Instruction::PutConst { register: 0, value: 555 };
        machine.code.push(Instruction::Proceed);
        machine.step().unwrap(); // TailCall for dummy
        machine.step().unwrap(); // Execute instruction at address 50
        assert_eq!(machine.registers[0], Some(Term::Const(555)));
        assert!(machine.environment_stack.is_empty());
    }

    // === Clause Management Instructions ===

    #[test]
    fn test_assert_clause_instruction() {
        let code = vec![
            Instruction::AssertClause { predicate: "p".to_string(), address: 123 },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap();
        // Check that the predicate table has the new clause.
        assert!(machine.predicate_table.get("p").unwrap().contains(&123));
        // If an index table for "p" exists, the clause should be appended.
        machine.index_table.insert("p".to_string(), {
            let mut map = HashMap::new();
            map.insert(vec![Term::Const(10)], vec![10]);
            map
        });
        let instr = Instruction::AssertClause { predicate: "p".to_string(), address: 456 };
        instr.execute(&mut machine).unwrap();
        let index_map = machine.index_table.get("p").unwrap();
        let clauses = index_map.get(&vec![Term::Const(10)]).unwrap();
        assert!(clauses.contains(&456));
    }

    #[test]
    fn test_retract_clause_instruction_success() {
        let code = vec![
            Instruction::AssertClause { predicate: "p".to_string(), address: 777 },
            Instruction::RetractClause { predicate: "p".to_string(), address: 777 },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // AssertClause
        machine.step().unwrap(); // RetractClause
        assert!(machine.predicate_table.get("p").unwrap().is_empty());
    }

    #[test]
    fn test_retract_clause_instruction_error() {
        let code = vec![
            Instruction::RetractClause { predicate: "p".to_string(), address: 888 },
        ];
        let mut machine = Machine::new(1, code);
        let result = machine.step();
        match result {
            Err(MachineError::PredicateNotFound(pred)) => assert_eq!(pred, "p".to_string()),
            _ => panic!("Expected PredicateNotFound error"),
        }
    }

    #[test]
    fn test_cut_instruction() {
        let code = vec![
            Instruction::Cut,
        ];
        let mut machine = Machine::new(1, code);
        // Simulate two choice points with different call levels.
        machine.control_stack.push(Frame { return_pc: 10 });
        machine.control_stack.push(Frame { return_pc: 20 });
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
        machine.step().unwrap(); // Execute Cut
        // Only choice points with call_level less than the current control stack length (2) remain.
        assert_eq!(machine.choice_stack.len(), 1);
        let remaining_cp = machine.choice_stack[0].clone();
        assert_eq!(remaining_cp.call_level, 1);
    }

    #[test]
    fn test_build_compound_instruction_success() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 10 },
            Instruction::PutConst { register: 1, value: 20 },
            Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
        ];
        let mut machine = Machine::new(3, code);
        machine.step().unwrap(); // PutConst reg0
        machine.step().unwrap(); // PutConst reg1
        machine.step().unwrap(); // BuildCompound
        let expected = Term::Compound("f".to_string(), vec![Term::Const(10), Term::Const(20)]);
        assert_eq!(machine.registers[2], Some(expected));
    }

    // === String Handling and Move ===

    #[test]
    fn test_put_str_instruction() {
        let code = vec![
            Instruction::PutStr { register: 0, value: "hello".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap();
        assert_eq!(machine.registers[0], Some(Term::Str("hello".to_string())));
    }

    #[test]
    fn test_get_str_instruction_success() {
        let code = vec![
            Instruction::PutStr { register: 0, value: "world".to_string() },
            Instruction::GetStr { register: 0, value: "world".to_string() },
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // PutStr
        machine.step().unwrap(); // GetStr
        assert_eq!(machine.registers[0], Some(Term::Str("world".to_string())));
    }

    #[test]
    fn test_move_instruction() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 77 },
            Instruction::Move { src: 0, dst: 1 },
        ];
        let mut machine = Machine::new(3, code);
        machine.step().unwrap(); // PutConst
        machine.step().unwrap(); // Move
        assert_eq!(machine.registers[1], Some(Term::Const(77)));
    }

    #[test]
    fn test_halt_instruction() {
        let code = vec![
            Instruction::Halt,
        ];
        let mut machine = Machine::new(1, code);
        machine.step().unwrap(); // Execute Halt
        // In this implementation, step() increments PC before executing,
        // so after a Halt instruction the PC becomes 1 (i.e. code.len()).
        assert_eq!(machine.pc, 1, "Expected PC to be 1 after executing Halt instruction");
    }
}
