#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;

    #[test]
    fn benchmark_path_inference() {
        // Construct a simple graph and a recursive path predicate.
        let code = vec![
            // Main Query: path(1,3)
            Instruction::PutConst { register: 0, value: 1 },   // X = 1
            Instruction::PutConst { register: 1, value: 3 },   // Y = 3
            Instruction::Call { predicate: "path".to_string() },
            Instruction::Proceed,
            // Facts for edge/2:
            // edge(1,2)
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::PutConst { register: 1, value: 2 },
            Instruction::Proceed,
            // edge(2,3)
            Instruction::PutConst { register: 0, value: 2 },
            Instruction::PutConst { register: 1, value: 3 },
            Instruction::Proceed,
            // edge(1,3)
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::PutConst { register: 1, value: 3 },
            Instruction::Proceed,
            // Clause for path/2: path(X,Y) :- edge(X,Y).
            Instruction::Call { predicate: "edge".to_string() },
            Instruction::Proceed,
            // Clause for path/2: path(X,Y) :- edge(X,Z), path(Z,Y).
            Instruction::PutVar { register: 2, var_id: 0, name: "Z".to_string() },
            Instruction::Call { predicate: "edge".to_string() },
            Instruction::Call { predicate: "path".to_string() },
            Instruction::Proceed,
        ];

        let mut machine = Machine::new(3, code);

        // Register the predicates.
        machine.register_predicate("edge".to_string(), 4);  // edge(1,2)
        machine.register_predicate("edge".to_string(), 7);  // edge(2,3)
        machine.register_predicate("edge".to_string(), 10); // edge(1,3)
        machine.register_predicate("path".to_string(), 13); // Clause 1 for path/2.
        machine.register_predicate("path".to_string(), 15); // Clause 2 for path/2.

        let start = std::time::Instant::now();
        machine.run().expect("Machine run should succeed");
        let duration = start.elapsed();
        
        println!("Path Inference Benchmark: Solution: X = {:?}, Y = {:?}", machine.registers[0], machine.registers[1]);
        println!("Path Inference Benchmark: Execution time: {:?}", duration);
        
        // Assert that a solution was found.
        assert!(machine.registers[0].is_some());
        assert!(machine.registers[1].is_some());
    }
}
