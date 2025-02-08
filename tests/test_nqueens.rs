#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;
    use std::time::Instant;

    /// This test simulates a solved 4–Queens problem.
    ///
    /// The solution is: [2, 4, 1, 3]
    ///
    /// We assume that:
    /// - register0 holds the board size (4)
    /// - register1 holds queen for row 1 (2)
    /// - register2 holds queen for row 2 (4)
    /// - register3 holds queen for row 3 (1)
    /// - register4 holds queen for row 4 (3)
    ///
    /// We build a list to represent the solution as:
    ///     cons(2, cons(4, cons(1, cons(3, nil))))
    ///
    /// Here, we represent nil as the constant 0 (placed in register9).
    /// The constructed list is stored in register5.
    #[test]
    fn test_nqueens_4_simulated() -> Result<(), MachineError> {
        println!("Running N-Queens simulated (N=4) test...");
        // Build a program that:
        // 1. Puts the board size and queen values.
        // 2. Constructs the solution list.
        // 3. Calls a (hypothetical) built-in predicate "print_solution"
        //    and then halts.
        let program = vec![
            // --- Setup: board size and queen assignments ---
            // Put N = 4 in register0.
            Instruction::PutConst { register: 0, value: 4 },
            // Queen for row 1 is 2 -> register1.
            Instruction::PutConst { register: 1, value: 2 },
            // Queen for row 2 is 4 -> register2.
            Instruction::PutConst { register: 2, value: 4 },
            // Queen for row 3 is 1 -> register3.
            Instruction::PutConst { register: 3, value: 1 },
            // Queen for row 4 is 3 -> register4.
            Instruction::PutConst { register: 4, value: 3 },
            // --- Setup: nil constant ---
            // Represent nil as the constant 0 in register9.
            Instruction::PutConst { register: 9, value: 0 },
            // --- Build the solution list ---
            // Build cons( queen4, nil ) into register8.
            // Queen4 is in register4 (value 3).
            Instruction::BuildCompound {
                target: 8,
                functor: "cons".to_string(),
                arg_registers: vec![4, 9],
            },
            // Build cons( queen3, register8 ) into register7.
            // Queen3 is in register3 (value 1).
            Instruction::BuildCompound {
                target: 7,
                functor: "cons".to_string(),
                arg_registers: vec![3, 8],
            },
            // Build cons( queen2, register7 ) into register6.
            // Queen2 is in register2 (value 4).
            Instruction::BuildCompound {
                target: 6,
                functor: "cons".to_string(),
                arg_registers: vec![2, 7],
            },
            // Build cons( queen1, register6 ) into register5.
            // Queen1 is in register1 (value 2).
            Instruction::BuildCompound {
                target: 5,
                functor: "cons".to_string(),
                arg_registers: vec![1, 6],
            },
            // --- Print and Halt ---
            // Call the built-in "print_solution" predicate.
            Instruction::Call { predicate: "print".to_string() },
            // Halt execution.
            Instruction::Halt,
        ];

        // Create a machine with at least 10 registers (we use registers 0 through 9).
        let mut machine = Machine::new(10, program);

        // Optionally, measure execution time.
        let start = Instant::now();
        machine.run()?;
        let duration = start.elapsed();
        println!("N-Queens simulated (N=4) execution time: {:?}", duration);

        // Construct the expected solution term:
        // cons(2, cons(4, cons(1, cons(3, nil))))
        let expected = Term::Compound("cons".to_string(), vec![
            Term::Const(2),
            Term::Compound("cons".to_string(), vec![
                Term::Const(4),
                Term::Compound("cons".to_string(), vec![
                    Term::Const(1),
                    Term::Compound("cons".to_string(), vec![
                        Term::Const(3),
                        Term::Const(0)  // nil is represented as 0
                    ])
                ])
            ])
        ]);

        // Check that register5 contains the expected solution.
        assert_eq!(machine.registers[5], Some(expected));

        Ok(())
    }
}
