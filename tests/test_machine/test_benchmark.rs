// tests/test_benchmark.rs

use lam::machine::arithmetic::Expression;
use lam::machine::instruction::Instruction;
use lam::machine::core::Machine;
use lam::machine::term::Term;
use std::time::Instant;

/// Benchmark test for the LAM.
///
/// This test repeatedly executes a small program that performs arithmetic evaluation
/// via the `ArithmeticIs` instruction. The program evaluates the expression (3 + 4) * 2,
/// which should yield 14. By running the program many times, we get an approximate measure
/// of the performance of the LAM’s core instruction dispatch and arithmetic evaluation.
///
/// Note:
/// - In a more advanced system, you might implement a recursive loop entirely within the LAM.
///   Here, we simply reinitialize and run the same small program many times.
/// - This benchmark serves as a baseline for performance before further optimizations.
#[test]
fn benchmark_arithmetic_is() {
    // Build the arithmetic expression: (3 + 4) * 2.
    let expr = Expression::Mul(
        Box::new(Expression::Add(Box::new(Expression::Const(3)), Box::new(Expression::Const(4)))),
        Box::new(Expression::Const(2))
    );

    // The program consists of one instruction that evaluates the expression and stores
    // the result in register 0.
    let code = vec![
        Instruction::ArithmeticIs { target: 0, expression: expr },
    ];

    // Set the number of iterations for the benchmark.
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        // Create a fresh machine for each iteration.
        let mut machine = Machine::new(1, code.clone());
        let _ = machine.run();
        // Verify correctness: register 0 should contain 14.
        assert_eq!(machine.registers[0], Some(Term::Const(14)));
    }
    let duration = start.elapsed();
    println!("Benchmark: Executed {} iterations in {:?}", iterations, duration);
}
