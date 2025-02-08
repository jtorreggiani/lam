#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_backtracking_variables() {
        let code = vec![
          // Step 0: PutVar reg0, variable X (id 0).
          Instruction::PutVar { register: 0, var_id: 0, name: "X".to_string() },
          // Step 1: Create a choice point with alternative branch at address 4.
          Instruction::Choice { alternative: 4 },
          // Step 2: First alternative: GetConst reg0, 100.
          Instruction::GetConst { register: 0, value: 100 },
          // Step 3: Force failure.
          Instruction::Fail,
          // Step 4: Second alternative: GetConst reg0, 300.
          Instruction::GetConst { register: 0, value: 300 },
        ];

        let mut machine = Machine::new(1, code);
        machine.run().expect("Machine run should succeed");

        // Verify that the union-find binding for variable 0 now yields Const(300).
        assert_eq!(machine.uf.resolve(&Term::Var(0)), Term::Const(300));
        // The choice stack should be empty.
        assert_eq!(machine.choice_stack.len(), 0);
    }
}
