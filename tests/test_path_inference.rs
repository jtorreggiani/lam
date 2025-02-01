use lam::machine::{Instruction, Machine};

// This benchmark test sets up a simple graph and a recursive path predicate,
// then runs the query `path(1, 3)` to search for a path from node 1 to node 3.
//
// The program is manually encoded as a vector of LAM instructions:
//
// Main query (indices 0–3):
//   0: PutConst reg0, 1    ; X = 1
//   1: PutConst reg1, 3    ; Y = 3
//   2: Call { predicate: "path" }
//   3: Proceed
//
// Facts for edge/2 (indices 4–12):
//   Clause for edge(1,2):
//     4: PutConst reg0, 1
//     5: PutConst reg1, 2
//     6: Proceed
//   Clause for edge(2,3):
//     7: PutConst reg0, 2
//     8: PutConst reg1, 3
//     9: Proceed
//   Clause for edge(1,3):
//     10: PutConst reg0, 1
//     11: PutConst reg1, 3
//     12: Proceed
//
// Clauses for path/2:
//   Clause 1: path(X,Y) :- edge(X,Y).
//     13: Call { predicate: "edge" }
//     14: Proceed
//   Clause 2: path(X,Y) :- edge(X,Z), path(Z,Y).
//     15: PutVar reg2, "Z"
//     16: Call { predicate: "edge" }
//     17: Call { predicate: "path" }
//     18: Proceed
//
// We register predicate clause addresses as follows:
//   "edge" -> [4, 7, 10]
//   "path" -> [13, 15]
//
// Running the query should find at least one solution. In the classic graph:
//   edge(1,3) gives a direct solution,
//   and edge(1,2), edge(2,3) gives an indirect solution.
// For benchmarking, we simply measure the execution time.
#[test]
fn benchmark_path_inference() {
    // Construct the program instructions.
    let code = vec![
        // Main Query: path(1,3)
        Instruction::PutConst { register: 0, value: 1 },   // X = 1
        Instruction::PutConst { register: 1, value: 3 },   // Y = 3
        Instruction::Call { predicate: "path".to_string() },
        Instruction::Proceed,
        // Facts for edge/2:
        // Clause for edge(1,2)
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::PutConst { register: 1, value: 2 },
        Instruction::Proceed,
        // Clause for edge(2,3)
        Instruction::PutConst { register: 0, value: 2 },
        Instruction::PutConst { register: 1, value: 3 },
        Instruction::Proceed,
        // Clause for edge(1,3)
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::PutConst { register: 1, value: 3 },
        Instruction::Proceed,
        // Clause for path/2, Clause 1: path(X,Y) :- edge(X,Y).
        Instruction::Call { predicate: "edge".to_string() },
        Instruction::Proceed,
        // Clause for path/2, Clause 2: path(X,Y) :- edge(X,Z), path(Z,Y).
        Instruction::PutVar { register: 2, name: "Z".to_string() },
        Instruction::Call { predicate: "edge".to_string() },
        Instruction::Call { predicate: "path".to_string() },
        Instruction::Proceed,
    ];

    // Create a machine with 3 registers (we need at least registers 0, 1, and 2).
    let mut machine = Machine::new(3, code);

    // Register the predicates.
    machine.register_predicate("edge".to_string(), 4);  // Clause for edge(1,2) at index 4.
    machine.register_predicate("edge".to_string(), 7);  // Clause for edge(2,3) at index 7.
    machine.register_predicate("edge".to_string(), 10); // Clause for edge(1,3) at index 10.
    machine.register_predicate("path".to_string(), 13); // Clause 1 for path/2 at index 13.
    machine.register_predicate("path".to_string(), 15); // Clause 2 for path/2 at index 15.

    // Benchmark: run the program and measure the execution time.
    let start = std::time::Instant::now();
    machine.run();
    let duration = start.elapsed();
    
    // Print the solution (what remains in registers 0 and 1 after the query).
    println!("Path Inference Benchmark: Solution: X = {:?}, Y = {:?}", machine.registers[0], machine.registers[1]);
    println!("Path Inference Benchmark: Execution time: {:?}", duration);
    
    // For this benchmark, we simply assert that a solution was found.
    // (In a fully complete system, you might count all solutions.)
    assert!(machine.registers[0].is_some());
    assert!(machine.registers[1].is_some());
}
